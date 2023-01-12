use crate::logic::logic_request::LogicRequest;
use crate::logic::storage_request::StorageRequest;
use cooplan_amqp_api::api::initialization_package::InitializationPackage;
use std::io::{Error, ErrorKind};
use std::time::Duration;

mod api;
pub mod config;
mod error;
mod logic;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Error> {
    match simple_logger::init() {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to initialize logger: {}", error),
            ));
        }
    }

    let api_file = match std::env::args().nth(1) {
        Some(api_file) => api_file,
        None => {
            return Err(Error::new(ErrorKind::InvalidInput, "no api file provided"));
        }
    };

    let config_file = match std::env::args().nth(2) {
        Some(config_file) => config_file,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "no config file provided",
            ))
        }
    };

    let config = match config::config::try_read_config(&config_file).await {
        Ok(config) => config,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to read config: {}", error),
            ));
        }
    };

    let (logic_request_sender, logic_request_receiver) =
        async_channel::bounded::<LogicRequest>(config.logic_requests_bound);

    let api_package =
        InitializationPackage::new(logic_request_sender, Box::new(api::registration::register));

    match cooplan_amqp_api::api::init::initialize(api_package).await {
        Ok(()) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to initialize api: {}", error),
            ));
        }
    }

    let (storage_request_sender, storage_request_receiver) =
        async_channel::bounded::<StorageRequest>(config.storage_requests_bound);

    match logic::init::initialize(
        config.logic_request_dispatch_instances,
        logic_request_receiver,
        storage_request_sender,
    )
    .await
    {
        Ok(()) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to initialize logic: {}", error),
            ));
        }
    }

    match storage::init::initialize(
        config.storage_request_dispatch_instances,
        storage_request_receiver,
    )
    .await
    {
        Ok(()) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to initialize storage: {}", error),
            ));
        }
    }

    std::thread::sleep(Duration::MAX);

    Ok(())
}
