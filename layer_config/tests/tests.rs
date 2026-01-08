use clap::Parser;
use layer_config::{LayerConfig, LayeredConfig};
use serde::{Deserialize, Serialize};
use std::env;

// Helper to convert &str args to Vec<String>, includes program name as first arg
fn args(args: &[&str]) -> Vec<String> {
    std::iter::once("test".to_string())
        .chain(args.iter().map(|s| s.to_string()))
        .collect()
}

// 1. Struct with `config` defined as a `String` and with `clap::Parser`.
#[derive(Parser, Deserialize, Serialize, Debug, PartialEq, LayerConfig)]
struct ConfigWithConfigField {
    #[clap(short, long, default_value = "default_config.yml")]
    config: String,
    #[clap(short, long, default_value = "default_user")]
    username: String,
    #[clap(short, long, default_value = "22")]
    port: i32,
}

#[test]
fn test_with_config_field() {
    let test_args = args(&["--config", "custom_config.yml", "--username", "user1", "--port", "80"]);
    let config: ConfigWithConfigField = ConfigWithConfigField::resolve_from(test_args).unwrap();
    assert_eq!(config.config, "custom_config.yml");
    assert_eq!(config.username, "user1");
    assert_eq!(config.port, 80);
}

// 2. Struct without `config` defined.
#[derive(Parser, Deserialize, Serialize, Debug, PartialEq, LayerConfig)]
struct ConfigWithoutConfigField {
    #[clap(short, long, default_value = "default_name")]
    name: String,
    #[clap(short, long, default_value = "30")]
    age: i32,
}

#[test]
fn test_without_config_field() {
    let test_args = args(&["--name", "Jane", "--age", "25"]);
    let config: ConfigWithoutConfigField = ConfigWithoutConfigField::resolve_from(test_args).unwrap();
    assert_eq!(config.name, "Jane");
    assert_eq!(config.age, 25);
}

// 3. Additional tests for environment variables and specified default values
#[derive(Parser, Deserialize, Serialize, Debug, PartialEq, LayerConfig)]
struct ConfigWithEnvAndDefaults {
    #[clap(short, long, default_value = "default_config.yml")]
    config: String,
    #[clap(short, long, default_value = "default_user")]
    username: String,
    #[clap(short, long, default_value = "22")]
    port: i32,
}

#[test]
fn test_env_and_defaults() {
    env::set_var("USERNAME", "env_user");
    let test_args = args(&["--port", "80"]); // Not setting `config` to test default value
    let config: ConfigWithEnvAndDefaults = ConfigWithEnvAndDefaults::resolve_from(test_args).unwrap();
    assert_eq!(config.config, "default_config.yml");
    assert_eq!(config.username, "env_user");
    assert_eq!(config.port, 80);
    env::remove_var("USERNAME");
}

#[test]
fn test_defaults_only() {
    let test_args = args(&[]);
    let config: ConfigWithoutConfigField = ConfigWithoutConfigField::resolve_from(test_args).unwrap();
    assert_eq!(config.name, "default_name");
    assert_eq!(config.age, 30);
}
