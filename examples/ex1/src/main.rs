#![cfg_attr(
    debug_assertions,
    allow(unused_imports, unused_variables, unused_mut, dead_code, unused_assignments)
)]

use clap::Parser;
use layer_config::LayerConfig;
use serde::{Deserialize, Serialize};

#[derive(Parser, Deserialize, Serialize, Debug, LayerConfig)]
struct Opts {
    #[clap(short, long, default_value = "config.yml")]
    config: String,

    #[clap(short, long, default_value = "John")]
    first_name: String,

    #[clap(short, long, default_value = "Doe")]
    last_name: String,

    #[clap(short, long, default_value = "42")]
    age: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::config()?;
    println!("opts={opts:?}");
    Ok(())
}
