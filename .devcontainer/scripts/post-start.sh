#!/usr/bin/env bash

set -eax

docker-login() {
  docker login -u "$DOCKER_USERNAME" -p "$DOCKER_TOKEN"
}

parallel --halt now,fail=1 \
    --linebuffer \
    -j0 ::: \
        "docker-login"