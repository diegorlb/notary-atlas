use serde::{Deserialize, Serialize};

use crate::domain::unaccent_and_lower;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Precision {
    Exact,
    Centroid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Location {
    pub formatted: String,
    pub normalized: String,

    pub lat: f64,
    pub lon: f64,

    pub precision: Precision,
}

impl Location {
    pub fn from_exact(address: String, lat: f64, lon: f64) -> Self {
        Self {
            normalized: unaccent_and_lower(&address),
            formatted: address,

            lat,
            lon,

            precision: Precision::Exact,
        }
    }

    pub fn from_centroid(address: String, lat: f64, lon: f64) -> Self {
        Self {
            normalized: unaccent_and_lower(&address),
            formatted: address,

            lat,
            lon,

            precision: Precision::Centroid,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Department {
    pub id: String,

    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NormalizedAddress {
    pub address: String,

    pub street: String,
    pub number: Option<String>,

    pub department_id: Option<String>,
    pub department_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalizedAddress {
    pub formatted: String,

    pub street: String,
    pub number: Option<String>,

    pub department: String,

    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notary {
    #[serde(skip)]
    pub search_key: String,

    pub tax_id: String,

    pub first_name: String,
    pub last_name: String,

    pub registry: String,
    pub status: String,
    pub city: String,
    pub address: String,

    pub license: License,

    pub emails: Emails,
    pub telephones: Telephones,

    pub district: District,
}

impl Notary {
    pub fn is_address_empty(&self) -> bool {
        self.address.trim().is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum License {
    Active(String),
    Retired(String),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Emails {
    pub public: Option<String>,
    pub alternative: Option<String>,
    pub institutional: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Telephones {
    pub primary: Option<String>,
    pub secondary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct District {
    pub id: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeocodedNotary {
    pub notary: Notary,
    pub geocoded: Option<Location>,
}

#[derive(Clone)]
pub struct NotariesGroupedIndices {
    pub location: Location,
    pub indices: Vec<usize>,
}

#[derive(Default, PartialEq, Eq, Clone)]
pub struct Filters {
    pub name: String,
    pub registry: String,
    pub location: String,

    pub precision: Option<Precision>,
}

impl Filters {
    pub const fn is_empty(&self) -> bool {
        self.name.is_empty()
            && self.registry.is_empty()
            && self.location.is_empty()
            && self.precision.is_none()
    }
}
