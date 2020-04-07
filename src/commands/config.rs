use cloudflare::endpoints::user::{
    GetUserDetails,
    GetUserTokenStatus,
};
#[allow(unused_imports)]
use cloudflare::framework::{
    apiclient::ApiClient,
    auth::Credentials,
    Environment,
    HttpApiClient,
    HttpApiClientConfig,
};
use failure;

use crate::{http, terminal};
use crate::config::{Config, Context, GlobalCredential, get_global_config_path};

pub fn save_credential(cred: &GlobalCredential) -> Result<(), failure::Error> {
    terminal::info("Validating credentials...");
    validate_credentials(&cred)?;

    let context = Context { name: "default".to_string(), credential: cred.to_owned() };
    let config = Config {
        current_context: "default".to_string(),
        contexts: vec![context],
    };

    let config_path = get_global_config_path()?;

    config.to_file(config_path.as_path())
}

// validate_credentials() checks the /user/tokens/verify endpoint (for API token)
// or /user endpoint (for global API key) to ensure provided credentials actually work.
// Source: https://github.com/cloudflare/wrangler/
pub fn validate_credentials(credential: &GlobalCredential) -> Result<(), failure::Error> {
    let client = HttpApiClient::new(
        Credentials::from(credential.to_owned()),
        HttpApiClientConfig::default(),
        Environment::Production,
    )?;

    match credential {
        GlobalCredential::Token { .. } => {
            match client.request(&GetUserTokenStatus {}) {
                Ok(success) => {
                    if success.result.status == "active" {
                        Ok(())
                    } else {
                        failure::bail!("Authentication check failed. Your token status is not active".to_owned())
                    }
                }
                Err(e) => failure::bail!(format!("Authentication check failed. Please make sure your API token is correct.\n{}", http::format_error(e, None)))
            }
        }
        GlobalCredential::GlobalKey { .. } => {
            match client.request(&GetUserDetails {}) {
                Ok(_) => Ok(()),
                Err(e) => failure::bail!(format!("Authentication check failed. Please make sure your email and global API key pair are correct. (https://developers.cloudflare.com/workers/quickstart/#global-api-key)\n{}", http::format_error(e, None))),
            }
        }
    }
}
