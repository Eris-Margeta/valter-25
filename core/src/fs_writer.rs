use anyhow::{Context, Result};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

pub struct FsWriter;

impl FsWriter {
    /// Ažurira jedno polje u YAML datoteci na siguran način (atomski).
    pub fn update_yaml_field(file_path: &Path, key: &str, value: &str) -> Result<()> {
        if !file_path.exists() {
            anyhow::bail!("File not found: {:?}", file_path);
        }

        // 1. Pročitaj postojeći sadržaj
        let content = fs::read_to_string(file_path)?;
        let mut yaml: Value = serde_yaml::from_str(&content)?;

        // 2. Ažuriraj vrijednost
        // Pokušaj pogoditi tip (broj vs string)
        let val_to_insert = if let Ok(num) = value.parse::<f64>() {
            Value::Number(serde_yaml::Number::from(num))
        } else if let Ok(bool_val) = value.parse::<bool>() {
            Value::Bool(bool_val)
        } else {
            Value::String(value.to_string())
        };

        if let Value::Mapping(ref mut map) = yaml {
            map.insert(Value::String(key.to_string()), val_to_insert);
        } else {
            anyhow::bail!("YAML Root is not a dictionary/mapping. Cannot update field.");
        }

        // 3. Atomski zapis (Write to temp -> Rename)
        // Ovo sprječava korupciju podataka ako nestane struje usred pisanja.
        let temp_path = file_path.with_extension("tmp");
        let new_content = serde_yaml::to_string(&yaml)?;

        fs::write(&temp_path, new_content)?;
        fs::rename(&temp_path, file_path)?;

        info!("FS Update: Set '{}' to '{}' in {:?}", key, value, file_path);
        Ok(())
    }

    /// Kreira novi projekt (Island) iz temelja
    pub fn create_island(
        root_dir: &str,
        name: &str,
        template_data: Vec<(String, String)>,
    ) -> Result<()> {
        // Sanitize name for folder
        let safe_name = name.replace(" ", "_").replace("/", "-");
        let project_path = Path::new(root_dir).join(&safe_name);

        if project_path.exists() {
            anyhow::bail!("Project folder already exists: {:?}", project_path);
        }

        fs::create_dir_all(&project_path).context("Failed to create project directory")?;

        // Izgradi početni YAML
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            Value::String("name".to_string()),
            Value::String(name.to_string()),
        );

        for (k, v) in template_data {
            map.insert(Value::String(k), Value::String(v));
        }

        // Dodaj timestamp
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        map.insert(Value::String("created_at".to_string()), Value::String(now));

        let yaml = Value::Mapping(map);
        let meta_path = project_path.join("meta.yaml");

        fs::write(&meta_path, serde_yaml::to_string(&yaml)?)?;

        info!("Created New Island: {}", name);
        Ok(())
    }
}
