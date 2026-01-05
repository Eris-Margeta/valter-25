use crate::config::Config;
use serde_json::{json, Value};
use anyhow::Result;

pub struct ToolGenerator;

impl ToolGenerator {
    pub fn generate_tools(config: &Config) -> Result<Value> {
        let mut tools = Vec::new();

        // 1. Generiraj alate za CLOUDS (Tablice)
        for cloud in &config.clouds {
            let tool_name = format!("get_{}", cloud.name.to_lowercase());
            let description = format!("Dohvati detalje za entitet '{}' iz baze.", cloud.name);
            
            let tool = json!({
                "type": "function",
                "function": {
                    "name": tool_name,
                    "description": description,
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": format!("UUID za {}", cloud.name)
                            }
                        },
                        "required": ["id"]
                    }
                }
            });
            tools.push(tool);
        }

        // 2. Generiraj alate za ISLANDS (Projekti i Agregacije)
        for island in &config.islands {
            for agg in &island.aggregations {
                let tool_name = format!("get_{}_{}", island.name.to_lowercase(), agg.name.to_lowercase());
                // FIX: Koristimo {:?} za ispis Enuma (Sum, Count...)
                let description = format!("Izračunaj '{}' ({:?}) za {}.", agg.name, agg.logic, island.name);
                
                let tool = json!({
                    "type": "function",
                    "function": {
                        "name": tool_name,
                        "description": description,
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "project_name": {
                                    "type": "string",
                                    "description": "Ime projekta (npr. 'Project Phoenix')"
                                }
                            },
                            "required": ["project_name"]
                        }
                    }
                });
                tools.push(tool);
            }
        }

        Ok(json!(tools))
    }
}

