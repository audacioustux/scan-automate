# Documentation

## The Architecture

- everything is designed to run on AWS EKS (with cluster autoscaler, ebs, metrics-server)
- a http api server acts as a proxy to the workflow engine (argo workflow)
- the workflow engine is responsible for running the scanners
- the scanners are implemented as steps in the workflow
- the api server can request for a scan progress status from the workflow engine
- the workflow collects the scan results, creates a pdf report, stores the report in s3, and sends an email with the report link

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

## Test Scan Workflow

### From Browser UI

_**Pre-requisite**_

- make sure scan-automate api server is accessible on localhost:4000
- (optional) port forward if necessary with `task api:port-forward`

_**Steps**_

- run `task api:ui:dev`
- open <http://localhost:3000>
  - enter url and email address, then Send Scan Request
  - a email should be sent to the email address with the scan confirmation link
  - click on the link to start the scan workflow
  - take not of the Scan ID
- use the scan id to request for a scan progress status

the status.phase and status.progress might be the most useful information in the progress status response

## Making Changes

### The API

in case it's necessary to make any change to the http api, crates/api is the place to make the change. The `ScanRequest` struct might be the most important struct to look at in order to extend the api.
> NOTE: make sure the changes are reflected in the `k8s/kustomize/scan-automate/webhook.yaml` trigger template parameters

currently the container image for the api is pushed to tanjim/scan-automate-api. To build and push the image to another registry, makes changes to `api:build-image` task in `taskfile.yaml`

### The Workflow

all the scanners are implemented as steps in Argo Workflow. The workflow is defined in `k8s/kustomize/scan-automate/workflow-template.yaml`. To add a new scanner, add a new step in the workflow template.

### The EKS Cluster

eks cluster is provisioned with terraform. The terraform code is in `terraform` directory. To make changes to the cluster, make changes to the terraform code and run `task eks:up` to apply the changes.

currently the cluster is set to use two node groups. `general` for platform components, `spot` for scanners / workflow pods.
