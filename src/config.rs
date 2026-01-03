use serde::Deserialize;
use std::collections::HashMap;
use anyhow::Result;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "DEFINITIONS")]
    pub definitions: Vec<Definition>,
    #[serde(rename = "VIEWS")]
    pub views: Option<Vec<View>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Definition {
    Cloud(CloudDef),
    Island(IslandDef),
}

#[derive(Debug, Deserialize)]
pub struct CloudDef {
    #[serde(rename = "CLOUD")]
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct IslandDef {
    #[serde(rename = "ISLAND")]
    pub name: String,
    pub path: String,
    pub meta_file: String,
    pub relations: Option<Vec<HashMap<String, String>>>,
}

#[derive(Debug, Deserialize)]
pub struct View {
    #[serde(rename = "GRAPH")]
    pub name: String,
    pub logic: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
