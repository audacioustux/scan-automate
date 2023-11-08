#!/usr/bin/env bash

set -eax

init-minikube() {
    # set the default profile
    minikube profile fncyber
    # minikube feature mounts ~/.minikube to devcontainer
    # so need to delete previous profile if it exists
    minikube delete
    # start minikube
    minikube start --cpus 6 --memory 6g --driver=docker --cni=false
    # use minikube's docker daemon
    local LINE='eval $(minikube docker-env)'
    local FILE=~/.zshrc
    grep -xqF -- "$LINE" "$FILE" || echo "$LINE" >> "$FILE"
}

# init-minikube