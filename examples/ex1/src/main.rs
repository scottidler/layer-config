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
    let opts = Opts::resolve()?;
    println!("opts={opts:?}");
    Ok(())
}
