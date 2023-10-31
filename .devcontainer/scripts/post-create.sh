#!/usr/bin/env bash

set -eax

# set the default profile
minikube profile fncyber
# minikube feature mounts ~/.minikube to devcontainer
# so need to delete previous profile if it exists
minikube delete
# start minikube
minikube start --cpus 6 --memory 6g --driver=docker --cni=false
# use minikube's docker daemon
LINE='eval $(minikube docker-env)'
FILE=~/.zshrc
grep -xqF -- "$LINE" "$FILE" || echo "$LINE" >> "$FILE"