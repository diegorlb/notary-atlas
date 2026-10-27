use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{Department, LocalizedAddress, simplify_for_match},
    utils::string_similarity,
};

#[derive(Serialize, Debug)]
struct GeorefDepartmentAPIRequest<'a> {
    #[serde(rename = "departamentos")]
    departments: Vec<GeorefDepartmentAPIRequestItem<'a>>,
}

#[derive(Serialize, Debug)]
struct GeorefDepartmentAPIRequestItem<'a> {
    #[serde(rename = "nombre")]
    name: String,

    #[serde(rename = "provincia")]
    province: &'a str,

    #[serde(rename = "orden")]
    order: &'a str,

    #[serde(rename = "aplanar")]
    flatten: bool,

    #[serde(rename = "max")]
    max_count: usize,
}

impl<'a> GeorefDepartmentAPIRequestItem<'a> {
    pub fn new(name: &'a str, max_count: usize) -> Self {
        Self {
            name: simplify_for_match(name),
            province: "Buenos Aires",
            order: "nombre",
            flatten: true,
            max_count,
        }
    }
}

#[derive(Deserialize, Debug)]
struct GeorefDepartmentAPIResponse {
    #[serde(rename = "resultados")]
    results: Vec<GeorefDepartmentAPIResponseItem>,
}

impl GeorefDepartmentAPIResponse {
    pub fn into_departments(self, names: &[String]) -> Vec<Option<Department>> {
        self.results
            .into_iter()
            .zip(names.iter())
            .map(|(item, name)| {
                let name = simplify_for_match(name);

                item.departments
                    .into_iter()
                    .map(|department| {
                        let department_name = simplify_for_match(&department.name);
                        let score = string_similarity(&name, &department_name);

                        (score, department)
                    })
                    .max_by(|(left, _), (right, _)| f64::total_cmp(left, right))
                    .map(|(_, department)| Department {
                        id: department.id,
                        lat: department.lat,
                        lon: department.lon,
                    })
            })
            .collect()
    }
}

#[derive(Deserialize, Debug)]
struct GeorefDepartmentAPIResponseItem {
    #[serde(rename = "departamentos")]
    departments: Vec<GeorefDepartmentAPIResponseDepartment>,
}

#[derive(Deserialize, Clone, Debug)]
struct GeorefDepartmentAPIResponseDepartment {
    id: String,

    #[serde(rename = "nombre")]
    name: String,

    #[serde(rename = "centroide_lat")]
    lat: f64,

    #[serde(rename = "centroide_lon")]
    lon: f64,
}

// ------------------------------------------------------------------------------------------------

#[derive(Serialize, Debug)]
struct GeorefAddressAPIRequest<'a> {
    #[serde(rename = "direcciones")]
    addresses: Vec<GeorefAddressAPIRequestItem<'a>>,
}

#[derive(Serialize, Debug)]
struct GeorefAddressAPIRequestItem<'a> {
    #[serde(rename = "direccion")]
    address: &'a str,

    #[serde(rename = "departamento", skip_serializing_if = "Option::is_none")]
    department: Option<&'a str>,

    #[serde(rename = "provincia")]
    province: &'a str,

    #[serde(rename = "orden")]
    order: &'a str,

    #[serde(rename = "aplanar")]
    flatten: bool,

    #[serde(rename = "max")]
    max_count: usize,
}

impl<'a> GeorefAddressAPIRequestItem<'a> {
    pub const fn new(address: &'a str, department: Option<&'a str>, max_count: usize) -> Self {
        Self {
            address,
            department,
            province: "Buenos Aires",
            order: "nombre",
            flatten: true,
            max_count,
        }
    }
}

#[derive(Deserialize, Debug)]
struct GeorefAddressAPIResponse {
    #[serde(rename = "resultados")]
    results: Vec<GeorefAddressAPIResponseItem>,
}

impl GeorefAddressAPIResponse {
    pub fn into_addresses(self) -> Vec<Vec<LocalizedAddress>> {
        self.results
            .into_iter()
            .map(|result| {
                result
                    .addresses
                    .into_iter()
                    .map(|raw_address| LocalizedAddress {
                        formatted: raw_address.formatted,
                        street: raw_address.street,
                        number: raw_address.number.map(|number| number.to_string()),
                        department: raw_address.department.unwrap_or_default(),
                        lat: raw_address.lat,
                        lon: raw_address.lon,
                    })
                    .collect()
            })
            .collect()
    }
}

#[derive(Deserialize, Debug)]
struct GeorefAddressAPIResponseItem {
    #[serde(rename = "direcciones")]
    addresses: Vec<GeorefAddressAPIResponseAddress>,
}

#[derive(Deserialize, Debug)]
struct GeorefAddressAPIResponseAddress {
    #[serde(rename = "nomenclatura")]
    formatted: String,

    #[serde(rename = "calle_nombre")]
    street: String,

    #[serde(rename = "altura_valor")]
    number: Option<usize>,

    #[serde(rename = "departamento_nombre")]
    department: Option<String>,

    #[serde(rename = "ubicacion_lat")]
    lat: Option<f64>,

    #[serde(rename = "ubicacion_lon")]
    lon: Option<f64>,
}

// ------------------------------------------------------------------------------------------------

const DEPARTMENT_API_URL: &str = "https://apis.datos.gob.ar/georef/api/v2.1/departamentos";
const ADDRESS_API_URL: &str = "https://apis.datos.gob.ar/georef/api/v2.1/direcciones";

#[derive(Default, Clone)]
pub struct GeorefClient {
    client: Client,
}

impl GeorefClient {
    pub async fn query_departments(
        &self,
        names: &[String],
        max_count: usize,
    ) -> Result<Vec<Option<Department>>> {
        if names.is_empty() {
            return Ok(Vec::new());
        }

        let request = GeorefDepartmentAPIRequest {
            departments: names
                .iter()
                .map(|name| GeorefDepartmentAPIRequestItem::new(name, max_count))
                .collect(),
        };

        let response = self
            .client
            .post(DEPARTMENT_API_URL)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;

            anyhow::bail!("Georef API failed with status {status}: {error_text}");
        }

        Ok(response
            .json::<GeorefDepartmentAPIResponse>()
            .await?
            .into_departments(names))
    }

    pub async fn query_addresses(
        &self,
        query: &[(String, Option<String>)],
        max_count: usize,
    ) -> Result<Vec<Vec<LocalizedAddress>>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let request = GeorefAddressAPIRequest {
            addresses: query
                .iter()
                .map(|(address, department)| {
                    GeorefAddressAPIRequestItem::new(address, department.as_deref(), max_count)
                })
                .collect(),
        };

        let response = self
            .client
            .post(ADDRESS_API_URL)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;

            anyhow::bail!("Georef API failed with status {status}: {error_text}");
        }

        Ok(response
            .json::<GeorefAddressAPIResponse>()
            .await?
            .into_addresses())
    }
}
