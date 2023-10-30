#!/usr/bin/env bash

set -ea

export REPO=`git remote get-url origin`
export ARGOCD_OPTS='--port-forward --port-forward-namespace argocd'

deploy-argocd(){
    echo "Deploying argocd..."
    kubectl create namespace argocd --dry-run=client -o yaml | kubectl apply -f -
    kubectl apply -k k8s/kustomize/argocd -n argocd

    echo "Waiting for all argocd pods to be ready..."
    kubectl wait --for=condition=ready pod \
        --all \
        -n argocd \
        --timeout=300s
}

login-argocd(){
    echo "Logging in to argocd..."
    local password=`kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d`

    argocd login --username admin --password $password
}

add-private-repos(){
    echo "Adding private repo..."
    local repo_url=($REPO)

    for repo in "${repo_url[@]}"
    do
        argocd repo add $repo --username ${GIT_USERNAME:?} --password ${GIT_TOKEN:?}
    done
}

deploy-argocd-apps(){
    echo "Deploying argocd apps..."
    kubectl apply --recursive -f k8s/apps
}

deploy-secrets(){
    echo "Deploying secrets..."
    local git=`kubectl create secret generic git-config --from-literal=username=${GIT_USERNAME:?} --from-literal=password=${GIT_TOKEN:?} --dry-run=client -o yaml`
    local docker=`kubectl create secret generic docker-config --from-file=$HOME/.docker/config.json --dry-run=client -o yaml`

    echo "$git" | kubectl apply -n argo -f -
    echo "$docker" | kubectl apply -n argo -f -
}

deploy-argocd
login-argocd
add-private-repos
deploy-argocd-apps

ebort -- deploy-secrets 2> /dev/null