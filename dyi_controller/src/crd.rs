use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "stable.dwk",
    version = "v1",
    kind = "DummySite",
    namespaced
)]
#[kube(printcolumn = r#"{"name": "Website URL", "type": "string", "jsonPath": ".spec.website_url"}"#)]
#[kube(printcolumn = r#"{"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp"}"#)]
pub struct DummySiteSpec {
    pub website_url: String,
}

impl DummySite {
    pub fn name(&self) -> crate::Result<String> {
        self.metadata
            .name
            .clone()
            .ok_or(crate::Error::MissingName)
    }

    pub fn namespace(&self) -> crate::Result<String> {
        self.metadata
            .namespace
            .clone()
            .ok_or(crate::Error::MissingNamespace)
    }
}