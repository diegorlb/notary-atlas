use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use itertools::Itertools as _;
use reqwest::ClientBuilder;
use serde::{Serialize, de::DeserializeOwned};
use tokio::task::spawn_blocking;

use crate::{
    domain::{
        Department, GeocodedNotary, LocalizedAddress, Location, NormalizedAddress, Notary,
        parse_address, simplify_for_match,
    },
    services::{
        api::{ColescbaClient, GeorefClient},
        storage::{load_json, save_json},
    },
    utils::string_similarity,
};

type DepartmentCache = HashMap<String, Department>;
type LocationCache = HashMap<String, Location>;

const MINIMUM_SCORE: f64 = 0.7;
const BATCH_SIZE: usize = 100;

const DEPARTMENTS_CACHE_PATH: &str = "data/departments_cache.json";
const GEOREF_CACHE_PATH: &str = "data/georef_cache.json";
const GEOCODED_NOTARIES_CACHE_PATH: &str = "data/notaries_cache.json";

async fn load_cache<T>(path: &str) -> T
where
    T: Send + Default + DeserializeOwned + 'static,
{
    let path = path.to_owned();
    spawn_blocking(move || match load_json(&path) {
        Ok(Some(cache)) => cache,
        _ => T::default(),
    })
    .await
    .unwrap_or_default()
}

async fn save_cache<T>(path: &str, data: &T) -> Result<()>
where
    T: Sync + Serialize,
{
    let value = serde_json::to_value(data)?;
    let path = path.to_owned();

    spawn_blocking(move || {
        let _ = save_json(path, &value);
    })
    .await?;

    Ok(())
}

fn normalize_notary(
    notary: Notary,
    department_cache: &DepartmentCache,
) -> (NormalizedAddress, Notary) {
    let (address, street, number) = parse_address(&notary.address);

    let department_name = notary.district.description.clone();

    let normalized_address = NormalizedAddress {
        address,

        street,
        number,

        department_id: department_cache
            .get(&department_name)
            .map(|department_data| &department_data.id)
            .cloned(),

        department_name,
    };

    (normalized_address, notary)
}

fn find_location(
    normalized: &NormalizedAddress,
    candidates: Vec<LocalizedAddress>,
) -> Option<Location> {
    let wanted_street = simplify_for_match(&normalized.street);
    let wanted_department = simplify_for_match(&normalized.department_name);

    candidates
        .into_iter()
        .rev()
        .filter(|candidate| candidate.lat.is_some() && candidate.lon.is_some())
        .map(|candidate| {
            let mut similarity_score =
                string_similarity(&wanted_street, &simplify_for_match(&candidate.street)) * 5.0;

            similarity_score += string_similarity(
                &wanted_department,
                &simplify_for_match(&candidate.department),
            ) * 1.0;

            similarity_score += match (&normalized.number, &candidate.number) {
                (Some(normalized_number), Some(candidate_number)) => {
                    let normalized_number: f64 = normalized_number.parse().unwrap_or_default();
                    let candidate_number: f64 = candidate_number.parse().unwrap_or_default();

                    let diff = (normalized_number - candidate_number).abs();

                    if diff > 100.0 {
                        0.0
                    } else {
                        (-diff / 20.0).exp()
                    }
                }

                (None, None) => 1.0,

                _ => 0.1,
            } * 2.0;

            (similarity_score / 8.0, candidate)
        })
        .filter(|(score, _)| *score >= MINIMUM_SCORE)
        .max_by(|left, right| f64::total_cmp(&left.0, &right.0))
        .map(|(_, best_match)| {
            Location::from_exact(
                best_match.formatted,
                best_match.lat.unwrap_or_default(),
                best_match.lon.unwrap_or_default(),
            )
        })
}

pub struct Geocoder {
    georef_client: GeorefClient,
    colescba_client: ColescbaClient,

    department_cache: DepartmentCache,
    georef_cache: LocationCache,
    geocoded_notaries_cache: Vec<GeocodedNotary>,
}

impl Geocoder {
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .user_agent("NotaryAtlas/1.0")
            .timeout(Duration::from_secs(16))
            .build()
            .unwrap_or_default();

        Self {
            georef_client: GeorefClient::new(client.clone()),
            colescba_client: ColescbaClient::new(client),

            department_cache: DepartmentCache::default(),
            georef_cache: LocationCache::default(),
            geocoded_notaries_cache: Vec::default(),
        }
    }

    pub async fn load_caches(&mut self) {
        self.department_cache = load_cache(DEPARTMENTS_CACHE_PATH).await;
        self.georef_cache = load_cache(GEOREF_CACHE_PATH).await;
        self.geocoded_notaries_cache = load_cache(GEOCODED_NOTARIES_CACHE_PATH).await;
    }

    pub async fn save_caches(&self) -> Result<()> {
        save_cache(DEPARTMENTS_CACHE_PATH, &self.department_cache).await?;
        save_cache(GEOREF_CACHE_PATH, &self.georef_cache).await?;
        save_cache(GEOCODED_NOTARIES_CACHE_PATH, &self.geocoded_notaries_cache).await?;

        Ok(())
    }

    pub async fn geocode(&mut self, force_refresh: bool) -> Result<Vec<GeocodedNotary>> {
        if !force_refresh && !self.geocoded_notaries_cache.is_empty() {
            log::info!(
                "Loaded {} notaries directly from local disk cache.",
                self.geocoded_notaries_cache.len()
            );

            return Ok(self.geocoded_notaries_cache.clone());
        }

        let original_notaries = self.colescba_client.query_notaries().await?;
        log::info!("Fetched {} notaries from API", original_notaries.len());

        let mut empty_addresses = original_notaries.len();
        let filtered_notaries: Vec<_> = original_notaries
            .into_iter()
            .filter(|notary| !notary.is_address_empty())
            .collect();
        empty_addresses -= filtered_notaries.len();

        if empty_addresses > 0 {
            log::info!("Skipped {empty_addresses} notaries with empty addresses");
        }

        self.query_departments(&filtered_notaries).await?;

        let normalized_notaries: Vec<_> = filtered_notaries
            .into_iter()
            .map(|notary| normalize_notary(notary, &self.department_cache))
            .collect();

        self.query_addresses(&normalized_notaries).await?;

        let mut geocoded_notaries = Vec::with_capacity(normalized_notaries.len());
        for (normalized_address, notary) in normalized_notaries {
            let address = normalized_address.address;

            let geocoded = self.georef_cache.get(&address).cloned();

            geocoded_notaries.push(GeocodedNotary { notary, geocoded });
        }

        self.save_caches().await?;

        Ok(geocoded_notaries)
    }

    async fn query_departments(&mut self, notaries: &[Notary]) -> Result<()> {
        let uncached_departments: Vec<_> = notaries
            .iter()
            .map(|notary| notary.district.description.as_str())
            .unique()
            .map(String::from)
            .filter(|department_name| !self.department_cache.contains_key(department_name))
            .collect();

        for (batch_index, batch) in uncached_departments.chunks(BATCH_SIZE).enumerate() {
            log::info!(
                "Processing department batch {} ({} departments)",
                batch_index + 1,
                batch.len()
            );

            let result = self.georef_client.query_departments(batch, 10).await?;
            for (department_name, department_data) in batch.iter().zip(result) {
                if let Some(department_data) = department_data {
                    self.department_cache
                        .insert(department_name.clone(), department_data);
                }
            }
        }

        Ok(())
    }

    async fn query_addresses(
        &mut self,
        normalized_notaries: &[(NormalizedAddress, Notary)],
    ) -> Result<()> {
        let uncached_addresses: Vec<_> = normalized_notaries
            .iter()
            .map(|(normalized_address, _)| normalized_address)
            .filter(|normalized_address| {
                !self.georef_cache.contains_key(&normalized_address.address)
            })
            .unique_by(|normalized_address| &normalized_address.address)
            .collect();

        if uncached_addresses.is_empty() {
            log::info!("All fetched addresses were already present in the cache.");

            return Ok(());
        }

        log::info!(
            "Found {} uncached addresses to geocode",
            uncached_addresses.len()
        );

        let mut using_centroid = 0;
        for (batch_index, batch) in uncached_addresses.chunks(BATCH_SIZE).enumerate() {
            let queries: Vec<_> = batch
                .iter()
                .map(|normalized_address| {
                    let address = normalized_address.address.clone();
                    let department_id = normalized_address.department_id.clone();

                    (address, department_id)
                })
                .collect();

            log::info!(
                "Processing batch {} ({} addresses)",
                batch_index + 1,
                queries.len()
            );

            let batch_responses = self
                .georef_client
                .query_addresses(&queries, 5000 / queries.len())
                .await?;

            for (normalized_address, response) in batch.iter().copied().zip(batch_responses) {
                let location = find_location(normalized_address, response).unwrap_or_else(|| {
                    using_centroid += 1;

                    let department_name = &normalized_address.department_name;
                    let (department_lat, department_lon) = self
                        .department_cache
                        .get(department_name)
                        .map_or((-34.6037, -58.3816), |department_data| {
                            (department_data.lat, department_data.lon)
                        });

                    Location::from_centroid(
                        normalized_address.address.clone(),
                        department_lat,
                        department_lon,
                    )
                });

                self.georef_cache
                    .insert(normalized_address.address.clone(), location);
            }
        }

        if using_centroid > 0 {
            log::info!("A total of {using_centroid} addresses are using the department centroid");
        }

        Ok(())
    }
}
