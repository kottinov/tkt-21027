use std::sync::Arc;

use dummysite_controller::prelude::*;
use futures::StreamExt;
use kube::{runtime::watcher::Config, runtime::Controller, Api, Client};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting DummySite controller");

    let client = Client::try_default().await?;
    let context = Arc::new(dummysite_controller::reconciler::Context::new(client.clone()));

    let dummysites = Api::<DummySite>::all(client);

    Controller::new(dummysites, Config::default())
        .run(
            Reconciler::reconcile,
            Reconciler::error_policy,
            context,
        )
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok((resource, _action)) => {
                    info!(resource = ?resource, "Reconciliation successful");
                }
                Err(error) => {
                    warn!(error = ?error, "Reconciliation error");
                }
            }
        })
        .await;

    Ok(())
}