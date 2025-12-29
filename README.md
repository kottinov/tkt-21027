# KubernetesSubmissions

## Exercises

### Chapter 2

- [1.1.](https://github.com/kottinov/tkt-21027/tree/1.1/log_output)
- [1.2.](https://github.com/kottinov/tkt-21027/tree/1.2/the_project)
- [1.3.](https://github.com/kottinov/tkt-21027/tree/1.3/log_output)
- [1.4.](https://github.com/kottinov/tkt-21027/tree/1.4/the_project)
- [1.5.](https://github.com/kottinov/tkt-21027/tree/1.5/the_project)
- [1.6.](https://github.com/kottinov/tkt-21027/tree/1.6/the_project)
- [1.7.](https://github.com/kottinov/tkt-21027/tree/1.7/log_output)
- [1.8.](https://github.com/kottinov/tkt-21027/tree/1.8/the_project)
- [1.9.](https://github.com/kottinov/tkt-21027/tree/1.9/)
- [1.10.](https://github.com/kottinov/tkt-21027/tree/1.10/log_output)
- [1.11.](https://github.com/kottinov/tkt-21027/tree/1.11/)
- [1.12.](https://github.com/kottinov/tkt-21027/tree/1.12/the_project)
- [1.13.](https://github.com/kottinov/tkt-21027/tree/1.13/the_project)

### Chapter 3

- [2.1.](https://github.com/kottinov/tkt-21027/tree/2.1/log_output)
- [2.2.](https://github.com/kottinov/tkt-21027/tree/2.2/)
- [2.3.](https://github.com/kottinov/tkt-21027/tree/2.3/)
- [2.4.](https://github.com/kottinov/tkt-21027/tree/2.4/)
- [2.5.](https://github.com/kottinov/tkt-21027/tree/2.5/log_output)
- [2.6.](https://github.com/kottinov/tkt-21027/tree/2.6/)
- [2.7.](https://github.com/kottinov/tkt-21027/tree/2.7/ping_pong)
- [2.8.](https://github.com/kottinov/tkt-21027/tree/2.8/todo_backend)
- [2.9.](https://github.com/kottinov/tkt-21027/tree/2.9/the_project)
- [2.10.](https://github.com/kottinov/tkt-21027/tree/2.10/assets/grafana_loki.png)

### Chapter 4

- [3.1.](https://github.com/kottinov/tkt-21027/tree/3.1/ping_pong)
- [3.2.](https://github.com/kottinov/tkt-21027/tree/3.2/)
- [3.3.](https://github.com/kottinov/tkt-21027/tree/3.3/)
- [3.4.](https://github.com/kottinov/tkt-21027/tree/3.4/)
- [3.5.](https://github.com/kottinov/tkt-21027/tree/3.5/the_project)
- [3.6.](https://github.com/kottinov/tkt-21027/tree/3.6/)
- [3.7.](https://github.com/kottinov/tkt-21027/tree/3.7/)
- [3.8.](https://github.com/kottinov/tkt-21027/tree/3.8/)
- [3.9.](https://github.com/kottinov/tkt-21027/tree/3.9/written_answers/3-9.md)
- [3.10.](https://github.com/kottinov/tkt-21027/tree/3.10/)
- [3.11.](https://github.com/kottinov/tkt-21027/tree/3.11/the_project)
- [3.12.](https://github.com/kottinov/tkt-21027/tree/3.12/assets/image.png)

### Chapter 5

- [4.1.](https://github.com/kottinov/tkt-21027/tree/4.1/)
- [4.2.](https://github.com/kottinov/tkt-21027/tree/4.2/assets)
- [4.3.](https://github.com/kottinov/tkt-21027/tree/4.3/)
- [4.3. assets](https://github.com/kottinov/tkt-21027/tree/4.3/assets)
- [4.4.](https://github.com/kottinov/tkt-21027/tree/4.4/ping_pong)
- [4.5.](https://github.com/kottinov/tkt-21027/tree/4.5/todo_backend)
- [4.6.](https://github.com/kottinov/tkt-21027/tree/4.6/broadcaster)
- [4.6. assets](https://github.com/kottinov/tkt-21027/tree/4.6/assets/image.png)
- [4.7.](https://github.com/kottinov/tkt-21027/tree/4.7/)
- [4.7. assets](https://github.com/kottinov/tkt-21027/tree/4.7/assets/image.png)
- [4.8.](https://github.com/kottinov/tkt-21027/tree/4.8/)
- [4.8. assets](https://github.com/kottinov/tkt-21027/tree/4.8/assets/image.png)
- [4.9.](https://github.com/kottinov/tkt-21027/tree/4.9/project)
- [4.10.](https://github.com/kottinov/tkt-21027-gitops) - GitOps repository (separate from code repo)

## GitOps Repository

Kubernetes manifests and configurations are maintained in a separate repository: [tkt-21027-gitops](https://github.com/kottinov/tkt-21027-gitops)

### Deployment Flow
1. Code changes pushed to this repository trigger GitHub Actions workflows
2. Workflows build and push Docker images to Google Artifact Registry
3. Workflows update kustomization files in the GitOps repository
4. ArgoCD detects changes in GitOps repository and syncs to Kubernetes clusters

# Chapter 6

- [5.1.](https://github.com/kottinov/tkt-21027/tree/5.1/dyi_controller)
- [5.2.](https://github.com/kottinov/tkt-21027/tree/5.2/assets/image.png)
- [5.3.](https://github.com/kottinov/tkt-21027/tree/5.3/log_output)