mod address;
mod filters;
mod models;

pub use address::{parse_address, simplify_for_match, unaccent_and_lower};
pub use filters::{apply_filters, group_notaries};
pub use models::{
    Department, District, Emails, Filters, GeocodedNotary, License, LocalizedAddress, Location,
    NormalizedAddress, NotariesGroupedIndices, Notary, Precision, Telephones,
};
