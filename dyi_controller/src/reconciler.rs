use std::sync::Arc;
use std::time::Duration;

use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Service;
use kube::{
    api::{Api, Patch, PatchParams},
    runtime::controller::Action,
    Client,
};
use tracing::{error, info};

use crate::{resources, DummySite, Error, Result};

#[derive(Clone)]
pub struct Context {
    pub client: Client,
}

impl Context {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

pub struct Reconciler;

impl Reconciler {
    pub async fn reconcile(dummysite: Arc<DummySite>, ctx: Arc<Context>) -> Result<Action> {
        let name = dummysite.name()?;
        let namespace = dummysite.namespace()?;

        info!(
            resource = %name,
            namespace = %namespace,
            url = %dummysite.spec.website_url,
            "Reconciling DummySite"
        );

        Self::reconcile_deployment(&dummysite, &ctx, &namespace, &name).await?;
        Self::reconcile_service(&ctx, &namespace, &name).await?;

        info!(
            resource = %name,
            namespace = %namespace,
            "Successfully reconciled DummySite"
        );

        Ok(Action::requeue(Duration::from_secs(300)))
    }

    async fn reconcile_deployment(
        dummysite: &DummySite,
        ctx: &Context,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let deployments: Api<Deployment> = Api::namespaced(ctx.client.clone(), namespace);
        let deployment = resources::build_deployment(dummysite, namespace, name);

        let patch_params = PatchParams::apply("dummysite-controller");
        let patch = Patch::Apply(&deployment);

        deployments
            .patch(&format!("dummysite-{}", name), &patch_params, &patch)
            .await?;

        info!(
            deployment = %format!("dummysite-{}", name),
            namespace = %namespace,
            "Created/Updated Deployment"
        );

        Ok(())
    }

    async fn reconcile_service(ctx: &Context, namespace: &str, name: &str) -> Result<()> {
        let services: Api<Service> = Api::namespaced(ctx.client.clone(), namespace);
        let service = resources::build_service(namespace, name);

        let patch_params = PatchParams::apply("dummysite-controller");
        let patch = Patch::Apply(&service);

        services
            .patch(&format!("dummysite-{}", name), &patch_params, &patch)
            .await?;

        info!(
            service = %format!("dummysite-{}", name),
            namespace = %namespace,
            "Created/Updated Service"
        );

        Ok(())
    }

    pub fn error_policy(
        dummysite: Arc<DummySite>,
        error: &Error,
        _ctx: Arc<Context>,
    ) -> Action {
        let name = dummysite
            .metadata
            .name
            .as_deref()
            .unwrap_or("<unknown>");

        error!(
            resource = %name,
            error = %error,
            "Reconciliation failed"
        );

        Action::requeue(Duration::from_secs(60))
    }
}