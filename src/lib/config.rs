static KEY: &str = "SPOTIFY_TOPS_CONFIG_FILE_DIR";
static FILE_NAME: &str = "spotifytopsconfig.toml";

use std::env;
use std::fs::File;
use std::io::{Read};
use std::path::PathBuf;

// extern crate dirs;
use dirs::home_dir;

// extern crate serde;
use serde::Deserialize;

lazy_static! {
    pub static ref CONFIG: Config = Default::default();
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_host_and_port: String,
    pub template_dir: String,
}

impl Config {
    pub fn default_config_file_dir() -> String {
        String::from(home_dir().unwrap_or_default().to_str().unwrap())
    }
}

impl Default for Config {
    fn default() -> Self {
        let config_file_dir = match env::var(KEY) {
            Ok(val) => val,
            Err(e) => Config::default_config_file_dir(),
        };

        let mut config_file_path = PathBuf::new();
        config_file_path.push(config_file_dir);
        config_file_path.push(FILE_NAME);

        let mut config_file = File::open(config_file_path).expect("Unable to open config file");
        let mut config_string = String::new();
        config_file
            .read_to_string(&mut config_string)
            .expect("Error reading config file");

        toml::from_str(config_string.as_str()).expect("Error parsing config file")
    }
}


#[cfg(test)]
mod tests {
    fn reads_config_from_default_path() {}
}
