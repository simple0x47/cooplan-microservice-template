use cooplan_amqp_api::config::openid_connect_config::OpenIdConnectConfig;
use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub openid_connect: OpenIdConnectConfig,
    pub logic_request_dispatch_instances: u16,
    pub logic_requests_bound: usize,
    pub storage_request_dispatch_instances: u16,
    pub storage_requests_bound: usize,
}

pub async fn try_read_config(config_file: &str) -> Result<Config, Error> {
    let config = match tokio::fs::read_to_string(config_file).await {
        Ok(config) => match serde_json::from_str::<Config>(config.as_str()) {
            Ok(config) => config,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::AutoConfigFailure,
                    format!("failed to deserialize config file's content: {}", error),
                ));
            }
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to read config file: {}", error),
            ));
        }
    };

    Ok(config)
}
