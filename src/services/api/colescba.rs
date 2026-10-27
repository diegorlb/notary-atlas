use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::{District, Emails, License, Notary, Telephones, unaccent_and_lower};

#[derive(Serialize, Default)]
struct NotaryAPIRequest<'a> {
    #[serde(rename = "nombres")]
    first_name: &'a str,

    #[serde(rename = "apellido")]
    last_name: &'a str,

    #[serde(rename = "carnet")]
    license: &'a str,

    #[serde(rename = "registro")]
    registry: &'a str,

    #[serde(rename = "domicilio")]
    address: &'a str,

    #[serde(rename = "localidad")]
    city: &'a str,

    #[serde(rename = "partido")]
    district: &'a str,
}

#[derive(Deserialize, Debug)]
struct NotaryAPIResponseItem {
    #[serde(rename = "cuitCuil")]
    tax_id: String,

    #[serde(rename = "nombres")]
    first_name: String,

    #[serde(rename = "apellido")]
    last_name: String,

    #[serde(rename = "condicion")]
    status: Option<String>,

    #[serde(rename = "localidad")]
    city: Option<String>,

    #[serde(rename = "domicilio")]
    address: Option<String>,

    #[serde(rename = "registro")]
    registry: Option<String>,

    #[serde(rename = "carnetActivo")]
    active_license: Option<String>,

    #[serde(rename = "carnetJubilado")]
    retired_license: Option<String>,

    #[serde(rename = "mailPublico")]
    public_email: Option<String>,

    #[serde(rename = "mailAlternativo")]
    alternative_email: Option<String>,

    #[serde(rename = "mailInstitucional")]
    institutional_email: Option<String>,

    #[serde(rename = "telefono1")]
    primary_telephone: Option<String>,

    #[serde(rename = "telefono2")]
    secondary_telephone: Option<String>,

    #[serde(rename = "partido")]
    district: NotaryAPIDistrict,
}

#[derive(Deserialize, Debug)]
struct NotaryAPIDistrict {
    #[serde(rename = "partidoId")]
    id: String,

    #[serde(rename = "descripcion")]
    description: String,
}

// ------------------------------------------------------------------------------------------------

const COLESCBA_API_URL: &str =
    "https://api.colescba.org.ar/wordpress/api/colescba/nomina-notarial-bsas/notarios/search";

#[derive(Default, Clone)]
pub struct ColescbaClient {
    client: Client,
}

impl ColescbaClient {
    pub async fn query_notaries(&self) -> Result<Vec<Notary>> {
        let request = NotaryAPIRequest::default();

        let response = self
            .client
            .post(COLESCBA_API_URL)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;

            anyhow::bail!("Georef API failed with status {status}: {error_text}");
        }

        Ok(response
            .json::<Vec<NotaryAPIResponseItem>>()
            .await?
            .into_iter()
            .map(|raw_notary| Notary {
                search_key: unaccent_and_lower(&format!(
                    "{} {} {} {} {}",
                    raw_notary.first_name,
                    raw_notary.last_name,
                    raw_notary.registry.as_deref().unwrap_or_default(),
                    raw_notary.city.as_deref().unwrap_or_default(),
                    raw_notary.district.description
                )),

                tax_id: raw_notary.tax_id,

                first_name: raw_notary.first_name.clone(),
                last_name: raw_notary.last_name.clone(),

                registry: raw_notary.registry.clone().unwrap_or_default(),
                status: raw_notary.status.unwrap_or_default(),
                city: raw_notary.city.clone().unwrap_or_default(),
                address: raw_notary.address.unwrap_or_default(),

                license: match (raw_notary.active_license, raw_notary.retired_license) {
                    (Some(active), _) => License::Active(active),
                    (_, Some(retired)) => License::Retired(retired),

                    _ => License::Unknown,
                },

                emails: Emails {
                    public: raw_notary.public_email,
                    alternative: raw_notary.alternative_email,
                    institutional: raw_notary.institutional_email,
                },

                telephones: Telephones {
                    primary: raw_notary.primary_telephone,
                    secondary: raw_notary.secondary_telephone,
                },

                district: District {
                    id: raw_notary.district.id,
                    description: raw_notary.district.description,
                },
            })
            .collect())
    }
}
