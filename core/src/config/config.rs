use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::{any::Any, error::Error};
use validator::Validate;

type LoaderFn = fn(&[u8]) -> Result<Box<dyn Any>, Box<dyn Error>>;
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub struct Config {
    loaders: HashMap<String, LoaderFn>,
}
impl Config {
    pub fn new() -> Self {
        let mut loaders: HashMap = HashMap::new();
        loaders.insert(".json", load_json::<T>);
        loaders.insert(".toml", load_toml::<T>);
        loaders.insert(".yaml", load_yaml::<T>);
        loaders.insert(".yml", load_yaml::<T>);
    }

    fn load_json<T: DeserializeOwned + 'static>(
        &mut self,
        data: &[u8],
    ) -> Result<T, Box<dyn Error>> {
        Ok(Box::new(serde_json::from_slice::<T>(data)?))
    }
    fn load_toml<T: DeserializeOwned + 'static>(
        &mut self,
        data: &[u8],
    ) -> Result<T, Box<dyn Error>> {
        Ok(serde_json::from_slice::<T>(data)?)
    }
    fn load_yaml<T: DeserializeOwned + 'static>(
        &mut self,
        data: &[u8],
    ) -> Result<T, Box<dyn Error>> {
        Ok(serde_yaml::<T>(data)?)
    }
    fn load_yml<T: DeserializeOwned + 'static>(
        &mut self,
        data: &[u8],
    ) -> Result<T, Box<dyn Error>> {
        Ok(serde_yaml::<T>(data)?)
    }

    pub fn load<T>(&self, file: &str, opts: &[Box<dyn Options>]) -> Result<Config, Box<dyn Error>>
    where
        T: DeserializeOwned + Validate,
    {
        let content = fs.read(file)?;
        let ext = Path::new(file)
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| ConfigError::UnsupportedFileType(ext.to_string()))?;

        let loader = self
            .loaders
            .get(ext)
            .ok_or_else(|| ConfigError::UnsupportedFileType(ext.to_string()))?;
        let opts = Options::new().use_env().build();

        let result = loader(content.as_bytes())?;
        let value =
            T::deserialize(&mut deserializer).map_err(|e| ConfigError::Parse(e.to_string()))?;

        value
            .validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))?;

        Ok(value)
    }
}

// 使用示例
#[derive(serde::Deserialize, Debug)]
struct MyConfig {
    name: String,
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loaders = init_loaders::<MyConfig>();
    let json_data = br#"{"name": "test", "port": 8080}"#;

    if let Some(loader) = loaders.get(".json") {
        let parsed = loader(json_data)?;
        if let Some(config) = parsed.downcast_ref::<MyConfig>() {
            println!("Loaded config: {:?}", config);
        }
    }
    Ok(())
}
