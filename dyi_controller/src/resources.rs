use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Container, ContainerPort, PodSpec, PodTemplateSpec, Service, ServicePort, ServiceSpec,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use std::collections::BTreeMap;

use crate::DummySite;

fn create_labels(name: &str) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();

    labels.insert("app".to_string(), format!("dummysite-{}", name));
    labels.insert("dummysite".to_string(), name.to_string());
    labels.insert("managed-by".to_string(), "dummysite-controller".to_string());
    labels
}

pub fn build_deployment(dummysite: &DummySite, namespace: &str, name: &str) -> Deployment {
    let labels = create_labels(name);
    let website_url = &dummysite.spec.website_url;

    Deployment {
        metadata: ObjectMeta {
            name: Some(format!("dummysite-{}", name)),
            namespace: Some(namespace.to_string()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(labels.clone()),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..Default::default()
                }),
                spec: Some(build_pod_spec(website_url)),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn build_pod_spec(website_url: &str) -> PodSpec {
    PodSpec {
        containers: vec![Container {
            name: "nginx".to_string(),
            image: Some("nginx:alpine".to_string()),
            ports: Some(vec![ContainerPort {
                container_port: 80,
                ..Default::default()
            }]),
            command: Some(vec!["/bin/sh".to_string()]),
            args: Some(vec![
                "-c".to_string(),
                format!(
                    "wget -O /usr/share/nginx/html/index.html '{}' && nginx -g 'daemon off;'",
                    website_url
                ),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn build_service(namespace: &str, name: &str) -> Service {
    let labels = create_labels(name);

    Service {
        metadata: ObjectMeta {
            name: Some(format!("dummysite-{}", name)),
            namespace: Some(namespace.to_string()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            selector: Some(labels),
            ports: Some(vec![ServicePort {
                port: 80,
                target_port: Some(IntOrString::Int(80)),
                ..Default::default()
            }]),
            type_: Some("ClusterIP".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_labels() {
        let labels = create_labels("example");
        assert_eq!(labels.get("app"), Some(&"dummysite-example".to_string()));
        assert_eq!(labels.get("dummysite"), Some(&"example".to_string()));
        assert_eq!(
            labels.get("managed-by"),
            Some(&"dummysite-controller".to_string())
        );
    }
}