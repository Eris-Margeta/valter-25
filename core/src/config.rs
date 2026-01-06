use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

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
