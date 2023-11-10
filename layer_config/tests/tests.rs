use clap::Parser;
use layer_config::LayerConfig;
use std::env;

// 1. Struct with `config` defined as a `String` and with `clap::Parser`.
#[derive(Debug, PartialEq, LayerConfig, clap::Parser)]
struct ConfigWithConfigField {
    #[clap(short = 'c', long, default_value = "default_config.yml")]
    config: String,
    #[clap(short = 'u', long, default_value = "default_user")]
    username: String,
    #[clap(short = 'p', long, default_value_t = 22)]
    port: i32,
}

#[test]
fn test_with_config_field() {
    let args = vec!["--config", "custom_config.yml", "--username", "user1", "--port", "80"];
    let config: ConfigWithConfigField = ConfigWithConfigField::resolve_from(args).unwrap();
    assert_eq!(config.config, "custom_config.yml");
    assert_eq!(config.username, "user1");
    assert_eq!(config.port, 80);
}

// 2. Struct without `config` defined, which should not require serde.
#[derive(Debug, PartialEq, LayerConfig, clap::Parser)]
struct ConfigWithoutConfigField {
    #[clap(short = 'n', long, default_value = "default_name")]
    name: String,
    #[clap(short = 'a', long, default_value_t = 30)]
    age: i32,
}

#[test]
fn test_without_config_field() {
    let args = vec!["--name", "Jane", "--age", "25"];
    let config: ConfigWithoutConfigField = ConfigWithoutConfigField::resolve_from(args).unwrap();
    assert_eq!(config.name, "Jane");
    assert_eq!(config.age, 25);
}

// 3. Struct with `config` defined but not as a `String` type.
#[derive(Debug, PartialEq, LayerConfig, clap::Parser)]
struct ConfigWithNonStringConfig {
    #[clap(short = 'i', long, default_value_t = 0)]
    config: i32,
    #[clap(short = 't', long, default_value = "default_text")]
    text: String,
    #[clap(short = 'b', long, default_value_t = false)]
    boolean: bool,
}

#[test]
#[should_panic(expected = "expected panic message")]
fn test_with_non_string_config() {
    let args = vec!["--config", "123", "--text", "value1", "--boolean"];
    let _config: ConfigWithNonStringConfig = ConfigWithNonStringConfig::resolve_from(args).unwrap();
}

// 4. Struct without `clap::Parser` derive, which should fail.
#[derive(Debug, PartialEq, LayerConfig)]
struct ConfigWithoutClapParser {
    #[clap(short = 'd', long, default_value = "default_data")]
    data: String,
    #[clap(short = 'q', long, default_value = "default_query")]
    query: String,
    #[clap(short = 'f', long, default_value_t = 1.0)]
    factor: f64,
}

#[test]
#[should_panic(expected = "expected panic message")]
fn test_without_clap_parser() {
    let args = vec!["--data", "data1", "--query", "query1", "--factor", "2.5"];
    let _config: ConfigWithoutClapParser = ConfigWithoutClapParser::resolve_from(args).unwrap();
}

// Additional tests for environment variables and specified default values
#[derive(Debug, PartialEq, LayerConfig, clap::Parser)]
struct ConfigWithEnvAndDefaults {
    #[clap(short = 'c', long, default_value = "default_config.yml")]
    config: String,
    #[clap(short = 'u', long, default_value = "default_user")]
    username: String,
    #[clap(short = 'p', long, default_value_t = 22)]
    port: i32,
}

#[test]
fn test_env_and_defaults() {
    env::set_var("USERNAME", "env_user");
    let args = vec!["--port", "80"]; // Not setting `config` to test default value
    let config: ConfigWithEnvAndDefaults = ConfigWithEnvAndDefaults::resolve_from(args).unwrap();
    assert_eq!(config.config, "default_config.yml");
    assert_eq!(config.username, "env_user");
    assert_eq!(config.port, 80);
    env::remove_var("USERNAME");
}

// Remember to add the necessary imports and dependencies in your Cargo.toml for clap and serde.
