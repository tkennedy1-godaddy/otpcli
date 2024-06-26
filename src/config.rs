use std::collections::HashMap;
use std::default::Default;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use crate::totp::TokenAlgorithm;
use crate::{TotpConfigError, TotpResult};
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    totp: HashMap<String, TotpOptions>,
}

impl Config {
    pub fn codes(&self) -> &HashMap<String, TotpOptions> {
        &self.totp
    }

    pub fn insert(&mut self, name: String, options: TotpOptions) {
        self.totp.insert(name, options);
    }

    pub fn remove(&mut self, name: &str) {
        self.totp.remove(name);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SecretLocation {
    #[serde(rename = "config")]
    Config,
    #[cfg(feature = "keychain")]
    #[serde(rename = "keychain")]
    KeyChain,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TotpOptions {
    storage: Option<SecretLocation>,
    secret: Option<String>,
    algorithm: Option<TokenAlgorithm>,
}

impl TotpOptions {
    pub fn storage(&self) -> Option<&SecretLocation> {
        self.storage.as_ref()
    }

    pub fn secret(&self) -> Option<&String> {
        self.secret.as_ref()
    }

    pub fn algorithm(&self) -> TokenAlgorithm {
        self.algorithm.unwrap_or(TokenAlgorithm::TotpSha1)
    }

    pub fn new_config_stored_secret(secret: String, algorithm: TokenAlgorithm) -> Self {
        TotpOptions {
            storage: Some(SecretLocation::Config),
            secret: Some(secret),
            algorithm: Some(algorithm),
        }
    }

    #[cfg(feature = "keychain")]
    pub fn new_keychain_stored_secret(algorithm: TokenAlgorithm) -> Self {
        TotpOptions {
            storage: Some(SecretLocation::KeyChain),
            secret: None,
            algorithm: Some(algorithm),
        }
    }
}

impl Config {
    pub fn lookup(&self, name: &str) -> TotpResult<&TotpOptions> {
        Ok(self
            .totp
            .get(name)
            .ok_or_else(|| TotpConfigError(format!("Unable to find config named '{}'", name)))?)
    }
}

pub fn load_config<P: AsRef<Path>>(config_dir: P) -> IoResult<Config> {
    let config_path: PathBuf = config_dir.as_ref().join("config.toml");

    let config: Config = if config_path.exists() {
        let config = std::fs::read_to_string(config_path)?;
        toml::from_str(&config).expect("Unable to read config as TOML")
    } else {
        Config::default()
    };

    Ok(config)
}

fn make_config_dir<P: AsRef<Path>>(config_dir: P) -> IoResult<()> {
    std::fs::create_dir_all(config_dir)
}

pub fn default_config_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Can't load users home directory");
    home_dir.join(".config").join("otpcli")
}

pub fn ensure_config_dir<P: AsRef<Path>>(config_dir: P) -> IoResult<()> {
    match std::fs::metadata(config_dir.as_ref()) {
        Err(_) => make_config_dir(config_dir.as_ref()),
        Ok(ref md) if !md.is_dir() => make_config_dir(config_dir.as_ref()),
        Ok(_) => Ok(()),
    }
}
