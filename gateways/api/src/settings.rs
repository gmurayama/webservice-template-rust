use eyre::Context;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize, Clone)]
pub struct Jaeger {
    pub host: String,
    pub port: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sampler_param: f64,
}

#[derive(serde::Deserialize, Clone)]
pub struct Application {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub environment: Environment,
    pub service_name: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub app: Application,
    pub jaeger: Jaeger,
}

pub fn get_config() -> eyre::Result<Settings> {
    let settings = config::Config::builder()
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("_"),
        )
        // App default settings
        .set_default("app.host", "127.0.0.1")?
        .set_default("app.port", 7000)?
        .set_default("app.environment", Environment::Development.as_str())?
        .set_default("app.service_name", "messaging")?
        // Jaeger default settings
        .set_default("jaeger.host", "127.0.0.1")?
        .set_default("jaeger.port", 6831)?
        .set_default("jaeger.sampler_param", 1.0)?
        .build()
        .wrap_err("error loading configuration from env variables")?;

    settings
        .try_deserialize::<Settings>()
        .wrap_err("error deserializing settings")
}

#[derive(serde::Deserialize, Clone, PartialEq, Eq)]
pub enum Environment {
    #[serde(rename = "development")]
    Development,
    #[serde(rename = "production")]
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
