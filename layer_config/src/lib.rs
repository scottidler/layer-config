pub use layer_config_derive::LayerConfig;

pub trait LayeredConfig: Sized {
    fn resolve() -> Result<Self, Box<dyn std::error::Error>>;
    fn resolve_from<T: AsRef<[String]>>(args: T) -> Result<Self, Box<dyn std::error::Error>>;
}
