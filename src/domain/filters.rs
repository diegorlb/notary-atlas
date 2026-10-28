use std::collections::HashMap;

use crate::{
    domain::{Filters, GeocodedNotary, NotariesGroupedIndices, unaccent_and_lower},
    utils::contains,
};

pub fn apply_filters(
    geocoded_notaries: &[GeocodedNotary],
    filters: &Filters,
) -> Vec<NotariesGroupedIndices> {
    if filters.is_empty() {
        return group_notaries(geocoded_notaries, 0..geocoded_notaries.len());
    }

    let filter_name = unaccent_and_lower(&filters.name);
    let filter_registry = unaccent_and_lower(&filters.registry);
    let filter_location = unaccent_and_lower(&filters.location);

    let filtered_indices =
        geocoded_notaries
            .iter()
            .enumerate()
            .filter_map(|(index, geocoded_notary)| {
                if let Some(precision) = &filters.precision {
                    let geocoded_notary_precision = geocoded_notary
                        .geocoded
                        .as_ref()
                        .map(|location| &location.precision);

                    if geocoded_notary_precision != Some(precision) {
                        return None;
                    }
                }

                let location_matches = geocoded_notary
                    .geocoded
                    .as_ref()
                    .is_some_and(|location| contains(&location.normalized, &filter_location));

                let search_key = &geocoded_notary.notary.search_key;

                let matches = search_key.contains(&filter_name)
                    && search_key.contains(&filter_registry)
                    && (location_matches || search_key.contains(&filter_location));

                if matches { Some(index) } else { None }
            });

    group_notaries(geocoded_notaries, filtered_indices)
}

pub fn group_notaries(
    geocoded_notaries: &[GeocodedNotary],
    indices: impl Iterator<Item = usize>,
) -> Vec<NotariesGroupedIndices> {
    let mut groups = HashMap::new();
    for index in indices {
        let notary = &geocoded_notaries[index];
        let Some(location) = notary.geocoded.as_ref() else {
            continue;
        };

        groups
            .entry(location.formatted.as_str())
            .or_insert_with(|| NotariesGroupedIndices {
                location: location.clone(),
                indices: Vec::new(),
            })
            .indices
            .push(index);
    }

    groups.into_values().collect::<Vec<_>>()
}
