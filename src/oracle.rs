use crate::config::{Config, Definition};
use serde_json::{json, Value};
use anyhow::Result;

pub struct ToolGenerator;

impl ToolGenerator {
    pub fn generate_tools(config: &Config) -> Result<Value> {
        let mut tools = Vec::new();

        // Generate tools for Clouds (Tables)
        for def in &config.definitions {
            if let Definition::Cloud(cloud) = def {
                let tool_name = format!("get_{}", cloud.name.to_lowercase());
                let description = format!("Retrieve information about {}", cloud.name);
                
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
                                    "description": format!("The UUID of the {}", cloud.name)
                                },
                                // Add dynamic fields if needed
                            },
                            "required": ["id"]
                        }
                    }
                });
                tools.push(tool);
            }
        }

        // Generate tools for Views
        if let Some(views) = &config.views {
            for view in views {
                let tool_name = format!("query_{}", view.name.to_lowercase());
                let description = format!("Analytical view: {}", view.logic);
                
                let tool = json!({
                    "type": "function",
                    "function": {
                        "name": tool_name,
                        "description": description,
                        "parameters": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    }
                });
                tools.push(tool);
            }
        }

        Ok(json!(tools))
    }
}
