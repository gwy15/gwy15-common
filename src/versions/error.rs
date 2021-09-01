#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error: {}", .0)]
    Network(#[from] reqwest::Error),

    #[error("Toml serde error: {}", .0)]
    TomlSerde(#[from] toml::de::Error),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;
