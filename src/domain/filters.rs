use std::collections::HashMap;

use crate::{
    domain::{Filters, GeocodedNotary, NotariesGroupedByLocation, unaccent_and_lower},
    utils::contains,
};

pub fn apply_filters(
    geocoded_notaries: &[GeocodedNotary],
    filters: &Filters,
) -> Vec<NotariesGroupedByLocation> {
    if filters.is_empty() {
        return group_notaries(geocoded_notaries.iter().cloned());
    }

    let filter_name = unaccent_and_lower(&filters.name);
    let filter_registry = unaccent_and_lower(&filters.registry);
    let filter_location = unaccent_and_lower(&filters.location);

    let filtered = geocoded_notaries
        .iter()
        .filter(|geocoded_notary| {
            if let Some(precision) = &filters.precision {
                let geocoded_notary_precision = geocoded_notary
                    .geocoded
                    .as_ref()
                    .map(|location| &location.precision);

                if geocoded_notary_precision != Some(precision) {
                    return false;
                }
            }

            let location_matches = geocoded_notary
                .geocoded
                .as_ref()
                .is_some_and(|location| contains(&location.normalized, &filter_location));

            let search_key = &geocoded_notary.notary.search_key;

            search_key.contains(&filter_name)
                && search_key.contains(&filter_registry)
                && (location_matches || search_key.contains(&filter_location))
        })
        .cloned();

    group_notaries(filtered)
}

pub fn group_notaries(
    geocoded_notaries: impl Iterator<Item = GeocodedNotary>,
) -> Vec<NotariesGroupedByLocation> {
    let mut groups = HashMap::new();
    for notary in geocoded_notaries {
        let Some(location) = notary.geocoded.clone() else {
            continue;
        };

        groups
            .entry(location.formatted.clone())
            .or_insert_with(|| NotariesGroupedByLocation {
                location,
                notaries: Vec::new(),
            })
            .notaries
            .push(notary.notary.clone());
    }

    groups.into_values().collect::<Vec<_>>()
}
