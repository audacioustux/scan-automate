# Documentation

## EKS Cluster

### Create EKS Cluster

_**Pre-requisite**_

- makes changes to k8s/cluster-autoscaler/cluster-autoscaler-autodiscover.yaml, change the IAM account-id
- login with aws cli

_**Steps**_

- run `task eks:up`

### Provision EKS Cluster

_**Pre-requisite**_

- make sure all the environment variables (.devcontainer/.env.example) are set properly, and is available to the shell

_**Steps**_

- run `task up`

