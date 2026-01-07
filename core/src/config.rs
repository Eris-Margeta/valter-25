use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

// ========================================================================= //
// STRUKTURE ZA PARSIRANJE `valter.config` YAML DATOTEKE
// ========================================================================= //

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(rename = "GLOBAL")]
    pub global: GlobalConfig,

    #[serde(rename = "CLOUDS")]
    pub clouds: Vec<CloudDefinition>,

    #[serde(rename = "ISLANDS")]
    pub islands: Vec<IslandDefinition>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalConfig {
    pub company_name: String,
    pub currency_symbol: String,
    pub locale: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    8000
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CloudDefinition {
    pub name: String,
    pub icon: String,
    pub fields: Vec<CloudField>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CloudField {
    pub key: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IslandDefinition {
    pub name: String,
    pub root_path: String,
    pub meta_file: String,
    #[serde(default)]
    pub relations: Vec<RelationRule>,
    #[serde(default)]
    pub aggregations: Vec<AggregationRule>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RelationRule {
    pub field: String,
    pub target_cloud: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AggregationRule {
    pub name: String,
    pub path: String,
    pub target_field: String,
    pub logic: AggregationLogic,
    #[serde(default)]
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AggregationLogic {
    Sum,
    Count,
    Average,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;

        if config.clouds.is_empty() {
            anyhow::bail!("Configuration must define at least one CLOUD.");
        }

        Ok(config)
    }
}

// ========================================================================= //
// MODUL ZA UPRAVLJANJE KONFIGURACIJOM IZ VARIJABLI OKRUŽENJA
// ========================================================================= //

pub mod env {
    use serde::Serialize;
    use std::collections::HashMap;
    use tracing::warn;

    /// Definira status i izvor konfiguracije.
    #[derive(Debug, Clone, Serialize)]
    #[serde(tag = "type", rename_all = "camelCase")]
    pub enum ConfigStatus {
        /// Koriste se vrijednosti ugrađene pri kompajliranju, jer je nadjačavanje onemogućeno.
        CompileTime,
        /// Koriste se ugrađene vrijednosti, jer je u runtime-u postavljena "ignore" zastavica.
        CompileTimeIgnored,
        /// Uspješno se koriste vrijednosti definirane u runtime okruženju.
        Runtime,
        /// Nadjačavanje je omogućeno, ali nedostaju neke obavezne varijable u runtime-u.
        RuntimeError {
            /// Eksplicitno preimenovanje za JSON kompatibilnost.
            #[serde(rename = "missingKeys")]
            missing_keys: Vec<String>,
        },
    }

    /// Sadrži konačne, razriješene vrijednosti konfiguracije koje aplikacija treba koristiti.
    #[derive(Debug, Clone)]
    pub struct EnvConfig {
        pub provider: &'static str,
        pub gemini_api_key: &'static str,
        pub model: &'static str,
        pub rpm: &'static str,
        pub search_api_key: &'static str,
        pub search_cx: &'static str,
        pub status: ConfigStatus,
    }

    impl EnvConfig {
        /// Inicijalizira konfiguraciju slijedeći strogu hijerarhiju definiranu u ADR-001.
        pub fn init() -> Self {
            // 1. Dohvati sve vrijednosti koje su ugrađene tijekom kompajliranja.
            let ct_provider = env!("VALTER_PROVIDER");
            let ct_gemini_api_key = env!("VALTER_GEMINI_API_KEY");
            let ct_model = env!("VALTER_MODEL");
            let ct_rpm = env!("VALTER_RPM");
            let ct_search_api_key = env!("VALTER_SEARCH_API_KEY");
            let ct_search_cx = env!("VALTER_SEARCH_CX");
            let ct_enable_overrides: bool =
                env!("ENV_VARIABLES_HIERARCHY_ENABLE_OVERRIDES_FROM_SYSTEM_DURING_RUNTIME")
                    .parse()
                    .unwrap_or(false);

            // Scenarij A: Nadjačavanje je onemogućeno u build-u. Uvijek koristi ugrađene vrijednosti.
            if !ct_enable_overrides {
                return Self {
                    provider: ct_provider,
                    gemini_api_key: ct_gemini_api_key,
                    model: ct_model,
                    rpm: ct_rpm,
                    search_api_key: ct_search_api_key,
                    search_cx: ct_search_cx,
                    status: ConfigStatus::CompileTime,
                };
            }

            // Scenarij B: Nadjačavanje je omogućeno. Slijedi stroga runtime provjera.
            warn!("Runtime overrides are enabled. Checking system environment variables.");

            // Korak B1: Provjeri "escape" varijablu.
            if std::env::var("VALTER_IGNORE_ENV_DURING_RUNTIME")
                .unwrap_or_default()
                .to_lowercase()
                == "true"
            {
                return Self {
                    provider: ct_provider,
                    gemini_api_key: ct_gemini_api_key,
                    model: ct_model,
                    rpm: ct_rpm,
                    search_api_key: ct_search_api_key,
                    search_cx: ct_search_cx,
                    status: ConfigStatus::CompileTimeIgnored,
                };
            }

            // Korak B2: "Sve ili ništa" provjera.
            let keys_to_check = [
                "VALTER_PROVIDER",
                "VALTER_GEMINI_API_KEY",
                "VALTER_MODEL",
                "VALTER_RPM",
                "VALTER_SEARCH_API_KEY",
                "VALTER_SEARCH_CX",
            ];

            let mut runtime_values = HashMap::new();
            let mut missing_keys = Vec::new();

            for &key in &keys_to_check {
                match std::env::var(key) {
                    Ok(val) if !val.is_empty() => {
                        runtime_values.insert(key, val);
                    }
                    _ => {
                        missing_keys.push(key.to_string());
                    }
                }
            }

            // Korak B3: Evaluacija stanja.
            if missing_keys.is_empty() {
                // SVE su pronađene. Koristi runtime vrijednosti.
                Self {
                    provider: Box::leak(
                        runtime_values.remove("VALTER_PROVIDER").unwrap().into_boxed_str(),
                    ),
                    gemini_api_key: Box::leak(
                        runtime_values.remove("VALTER_GEMINI_API_KEY").unwrap().into_boxed_str(),
                    ),
                    model: Box::leak(
                        runtime_values.remove("VALTER_MODEL").unwrap().into_boxed_str(),
                    ),
                    rpm: Box::leak(runtime_values.remove("VALTER_RPM").unwrap().into_boxed_str()),
                    search_api_key: Box::leak(
                        runtime_values.remove("VALTER_SEARCH_API_KEY").unwrap().into_boxed_str(),
                    ),
                    search_cx: Box::leak(
                        runtime_values.remove("VALTER_SEARCH_CX").unwrap().into_boxed_str(),
                    ),
                    status: ConfigStatus::Runtime,
                }
            } else {
                // Barem jedna nedostaje. Koristi ugrađene vrijednosti i prijavi grešku.
                Self {
                    provider: ct_provider,
                    gemini_api_key: ct_gemini_api_key,
                    model: ct_model,
                    rpm: ct_rpm,
                    search_api_key: ct_search_api_key,
                    search_cx: ct_search_cx,
                    status: ConfigStatus::RuntimeError { missing_keys },
                }
            }
        }
    }
}
