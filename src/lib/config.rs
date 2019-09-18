// TODO(lmp) can this be a macro?
static KEY: &str = "SPOTIFY_TOPS_CONFIG_FILE_DIR";
static CLIENT_ID_KEY: &str = "SPOTIFY_TOPS_CLIENT_ID";
static CLIENT_SECRET_KEY: &str = "SPOTIFY_TOPS_CLIENT_SECRET";
static REDIRECT_HOST_AND_PORT_KEY: &str = "SPOTIFY_TOPS_REDIRECT_HOST_AND_PORT";
static TEMPLATE_DIR_KEY: &str = "SPOTIFY_TOPS_TEMPLATE_DIR";
static LISTEN_ADDR_KEY: &str = "SPOTIFY_TOPS_LISTEN_ADDR";
static LISTEN_PORT_KEY: &str = "PORT";
static STATIC_DIR_KEY: &str = "SPOTIFY_TOPS_STATIC_DIR";
static FILE_NAME: &str = "spotifytopsconfig.toml";

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use dirs::home_dir;

use serde::Deserialize;

lazy_static! {
    pub static ref CONFIG: Config = Default::default();
}

#[derive(Deserialize, Debug)]
pub struct DeserializedConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_host_and_port: Option<String>,
    pub template_dir: Option<String>,
    pub listen_addr: Option<String>,
    pub listen_port: Option<String>,
    pub static_dir: Option<String>,
}

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_host_and_port: String,
    pub template_dir: String,
    pub listen_addr: String,
    pub listen_port: String,
    pub static_dir: String,
}

impl Config {
    pub fn default_config_file_dir() -> String {
        String::from(home_dir().unwrap_or_default().to_str().unwrap())
    }
}

fn env_var_deserialized_or_default(
    env_var_name: &str,
    deserialized: Option<String>,
    default: &str,
) -> String {
    let env_result = env::var(env_var_name);
    if env_result.is_ok() {
        return env_result.unwrap();
    }

    if deserialized.is_some() {
        return deserialized.unwrap();
    }

    String::from(default)
}

impl Default for Config {
    fn default() -> Self {
        let config_file_dir = match env::var(KEY) {
            Ok(val) => val,
            Err(_) => Config::default_config_file_dir(),
        };

        let mut config_file_path = PathBuf::new();
        config_file_path.push(config_file_dir);
        config_file_path.push(FILE_NAME);

        let config = if let Ok(mut config_file) = File::open(config_file_path) {
            let mut config_string = String::new();
            config_file
                .read_to_string(&mut config_string)
                .expect("Error reading config file");

            toml::from_str(config_string.as_str()).unwrap()
        } else {
            DeserializedConfig {
                client_id: None,
                client_secret: None,
                redirect_host_and_port: None,
                template_dir: None,
                listen_addr: None,
                listen_port: None,
                static_dir: None,
            }
        };

        Config {
            client_id: env_var_deserialized_or_default(CLIENT_ID_KEY, config.client_id, ""),
            client_secret: env_var_deserialized_or_default(
                CLIENT_SECRET_KEY,
                config.client_secret,
                "",
            ),
            redirect_host_and_port: env_var_deserialized_or_default(
                REDIRECT_HOST_AND_PORT_KEY,
                config.redirect_host_and_port,
                "",
            ),
            template_dir: env_var_deserialized_or_default(
                TEMPLATE_DIR_KEY,
                config.template_dir,
                "",
            ),
            listen_addr: env_var_deserialized_or_default(LISTEN_ADDR_KEY, config.listen_addr, ""),
            listen_port: env_var_deserialized_or_default(LISTEN_PORT_KEY, config.listen_port, ""),
            static_dir: env_var_deserialized_or_default(STATIC_DIR_KEY, config.static_dir, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    //fn reads_config_from_default_path() {}
}
