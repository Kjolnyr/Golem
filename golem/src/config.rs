use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub bot_name: String,
    pub bot_token: String,
    pub bot_owners: Vec<u64>,
    pub proc_path: String,
    pub vars_path: String
}

impl Config {
    pub fn load(config_path: &str) -> std::io::Result<Self> {
        let mut config_file = File::open(config_path)?;
        let mut content = String::new();

        config_file.read_to_string(&mut content)?;

        let config: Config = serde_yaml::from_str(&content).unwrap();

        Ok(config)
    }
}
