pub use semver::Version;

mod error;
pub use error::{Error, Result};

use log::*;
use reqwest::Client;
use std::{collections::HashMap, time::Duration};

const URLS: &[&str] = &[
    "https://raw.githubusercontent.com/gwy15/versions/main/versions.toml",
    "https://raw.fastgit.org/gwy15/versions/main/versions.toml",
];

pub async fn get_versions(client: &Client, url: &str) -> Result<HashMap<String, Version>> {
    let rsp = client.get(url).send().await?;
    let text = rsp.text().await?;
    let versions = toml::from_str(&text)?;
    Ok(versions)
}

pub async fn get_version(identifier: impl AsRef<str>) -> Result<Option<Version>> {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(3))
        .build()?;
    for (idx, url) in URLS.iter().enumerate() {
        debug!("using url {} to retrieve version info...", url);
        match get_versions(&client, url).await {
            Ok(versions) => return Ok(versions.get(identifier.as_ref()).cloned()),
            Err(e) => {
                debug!("get version failed: {:?}", e);
                if idx == URLS.len() - 1 {
                    error!("get version ran out all urls.");
                    return Err(e);
                } else {
                    warn!(
                        "failed to get versions from url {}, will try fallbacks.",
                        url
                    );
                    continue;
                }
            }
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let content = r#"
test = "0.1.0"
alpha = "0.1.2-alpha"
beta = "0.2.3-beta.1"
"#;
        let _s: HashMap<String, Version> = toml::from_str(content).unwrap();
    }

    #[tokio::test]
    async fn test_get_version_non_exist() {
        pretty_env_logger::try_init_timed().ok();
        assert_eq!(get_version("").await.unwrap(), None);
        assert_eq!(get_version("non_exist").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_get_version() {
        let test = get_version("test").await.unwrap().unwrap();
        let alpha = get_version("alpha").await.unwrap().unwrap();
        let beta = get_version("beta").await.unwrap().unwrap();
        assert_eq!(test, "0.1.0".parse().unwrap());
        assert_eq!(alpha, "0.1.2-alpha".parse().unwrap());
        assert_eq!(beta, "0.2.3-beta.1".parse().unwrap());
        assert!(beta > alpha);
    }
}
