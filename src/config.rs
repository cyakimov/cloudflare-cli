#[cfg(not(target_os = "windows"))]
use std::fs::File;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
#[cfg(not(target_os = "windows"))]
use std::path::PathBuf;

use cloudflare::framework::auth::Credentials;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum GlobalCredential {
    Token { api_token: String },
    GlobalKey { email: String, api_key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub name: String,
    pub credential: GlobalCredential,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub current_context: String,
    pub contexts: Vec<Context>,
}

// set the permissions on the dir, we want to avoid that other user reads to file
#[cfg(not(target_os = "windows"))]
pub fn set_file_mode(file: &PathBuf) {
    File::open(&file)
        .unwrap()
        .set_permissions(PermissionsExt::from_mode(0o600))
        .expect("could not set permissions on file");
}

pub fn get_global_config_path() -> Result<PathBuf, failure::Error> {
    let home_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cflare");
    let config_path = home_dir.join("default.toml");
    Ok(config_path)
}

impl Config {
    pub fn to_file(&self, config_path: &Path) -> Result<(), failure::Error> {
        let toml = toml::to_string(self)?;

        fs::create_dir_all(&config_path.parent().unwrap())?;
        fs::write(&config_path, toml)?;

        // set permissions on the file
        #[cfg(not(target_os = "windows"))]
            set_file_mode(&config_path.to_path_buf());

        Ok(())
    }

    pub fn from_file(config_path: PathBuf) -> Result<Self, failure::Error> {
        let config_str = config_path
            .to_str()
            .expect("global config path should be a string");

        if !config_path.exists() {
            failure::bail!(
                "config path does not exist {}. Try running `cflare config`",
                config_str
            )
        }

        match fs::read_to_string(config_path) {
            Ok(c) => {
                match toml::from_str(&c) {
                    Ok(conf) => Ok(conf),
                    _ => failure::bail!("invalid config format")
                }
            }
            _ => failure::bail!("error while reading config file")
        }
    }
}

impl From<GlobalCredential> for Credentials {
    fn from(user: GlobalCredential) -> Credentials {
        match user {
            GlobalCredential::Token { api_token } => Credentials::UserAuthToken { token: api_token },
            GlobalCredential::GlobalKey { email, api_key } => Credentials::UserAuthKey {
                key: api_key,
                email,
            },
        }
    }
}
