use crate::error::{Error, ErrorKind};
use crate::logic::storage_request::StorageRequest;
use async_channel::Receiver;
use cooplan_mongodb::config::mongodb_config;
use cooplan_mongodb::connection_manager::MongoDbConnectionManager;

pub async fn initialize(
    concurrent_dispatchers: u16,
    request_receiver: Receiver<StorageRequest>,
) -> Result<(), Error> {
    let config = match mongodb_config::try_generate_config().await {
        Ok(config) => config,
        Err(error) => return Err(Error::new(ErrorKind::AutoConfigFailure, error.message)),
    };

    let mongodb_connection_manager = match MongoDbConnectionManager::try_new(config) {
        Ok(mongodb_connection_manager) => mongodb_connection_manager,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::StorageFailure,
                format!("failed to initialize mongodb connection manager: {}", error),
            ))
        }
    };

    let client = mongodb_connection_manager.client();

    for _ in 0..concurrent_dispatchers {
        let mongodb_request_dispatch =
            crate::storage::mongodb_request_dispatch::MongoDbRequestDispatch::new(
                client.clone(),
                request_receiver.clone(),
            );

        tokio::spawn(mongodb_request_dispatch.run());
    }

    Ok(())
}
