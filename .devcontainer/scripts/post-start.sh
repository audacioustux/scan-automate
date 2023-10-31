#!/usr/bin/env bash

set -eax

docker-login() {
  docker login -u "$DOCKER_USERNAME" -p "$DOCKER_TOKEN"
}

tunnel-minikube() {
  minikube tunnel --bind-address "0.0.0.0"
}

minikube status || minikube start 

parallel --halt now,fail=1 \
    --linebuffer \
    -j0 ::: \
        "docker-login" \
        "tunnel-minikube" 