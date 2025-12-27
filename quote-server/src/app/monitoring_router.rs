use crate::app::client_address::ClientAddress;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::instrument;
use tracing_log::log::info;

#[derive(Default)]
pub(crate) struct MonitoringRouter {
    monitoring_router: RwLock<HashMap<ClientAddress, Sender<()>>>,
}

impl MonitoringRouter {
    #[instrument(
        name = "Add monitoring route",
        skip(self, monitoring_tx),
        fields(address)
    )]
    pub(crate) fn add_route(
        &self,
        address: ClientAddress,
        monitoring_tx: Sender<()>,
    ) -> Result<(), MonitoringRouterError> {
        let mut monitoring_router = self
            .monitoring_router
            .write()
            .map_err(|e| MonitoringRouterError::RwLockPoisoned(e.to_string()))?;
        monitoring_router.insert(address, monitoring_tx);
        Ok(())
    }

    #[instrument(name = "Send ping", skip(self), fields(address))]
    pub(crate) fn send_ping(&self, address: &ClientAddress) -> Result<(), MonitoringRouterError> {
        let mut is_alive = true;

        {
            let lock = self
                .monitoring_router
                .read()
                .map_err(|e| MonitoringRouterError::RwLockPoisoned(e.to_string()))?;
            if let Some(monitoring_tx) = lock.get(address)
                && monitoring_tx.send(()).is_err()
            {
                is_alive = false;
            }
        }

        if !is_alive {
            self.delete(address)?;
        }

        Ok(())
    }

    #[instrument(name = "Delete monitoring route", skip(self), fields(address))]
    pub(crate) fn delete(&self, address: &ClientAddress) -> Result<(), MonitoringRouterError> {
        let mut lock = self
            .monitoring_router
            .write()
            .map_err(|e| MonitoringRouterError::RwLockPoisoned(e.to_string()))?;
        lock.remove(address);
        info!("Removed monitoring route for address: {}", address);
        println!("Removed monitoring route for address: {}", address);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MonitoringRouterError {
    #[error("Failed to get lock: {0}")]
    RwLockPoisoned(String),
}
