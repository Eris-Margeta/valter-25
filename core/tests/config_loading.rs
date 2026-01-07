use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use valter_core::config::Config;

#[test]
fn test_load_valid_config() {
    let dir = tempdir().expect("Failed to create temp dir");
    let config_path = dir.path().join("valter.test.config");
    let mut file = File::create(&config_path).expect("Failed to create test config file");

    let yaml_content = r#"
GLOBAL:
  company_name: "Test Corp"
  currency_symbol: "$"
  locale: "en_US"
  port: 8080
CLOUDS:
  - name: "Client"
    icon: "briefcase"
    fields:
      - key: "name"
        type: "string"
        required: true
ISLANDS: []
"#;

    write!(file, "{}", yaml_content).expect("Failed to write to test config");

    let config_result = Config::load(config_path.to_str().unwrap());

    assert!(config_result.is_ok());
    let config = config_result.unwrap();
    assert_eq!(config.global.company_name, "Test Corp");
    assert_eq!(config.clouds.len(), 1);
    assert_eq!(config.clouds[0].name, "Client");
}

#[test]
fn test_load_config_missing_clouds() {
    let dir = tempdir().expect("Failed to create temp dir");
    let config_path = dir.path().join("valter.test.config");
    let mut file = File::create(&config_path).expect("Failed to create test config file");

    let yaml_content = r#"
GLOBAL:
  company_name: "Test Corp"
  currency_symbol: "$"
  locale: "en_US"
  port: 8080
CLOUDS: [] # Prazna lista
ISLANDS: []
"#;

    write!(file, "{}", yaml_content).expect("Failed to write to test config");

    let config_result = Config::load(config_path.to_str().unwrap());

    assert!(config_result.is_err());
    assert!(config_result
        .unwrap_err()
        .to_string()
        .contains("must define at least one CLOUD"));
}
