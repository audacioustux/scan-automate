#!/usr/bin/env bash

set -ea

export REPO=`git remote get-url origin`

deploy-argocd(){
    echo "Deploying argocd..."
    kubectl create namespace argocd --dry-run=client -o yaml | kubectl apply -f -
    kubectl apply -k k8s/kustomize/argocd -n argocd

    echo "Waiting for all argocd pods to be ready..."
    sleep 5
    kubectl wait --for=condition=ready pod \
        --all \
        -n argocd \
        --timeout=300s

    kubectl patch svc argocd-server -n argocd -p '{"spec": {"type": "LoadBalancer"}}'
}

login-argocd(){
    echo "Logging in to argocd..."
    local password=`kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d`
    local argocd_server=`kubectl get svc argocd-server -n argocd -o jsonpath='{.status.loadBalancer.ingress[0].hostname}'`

    argocd login $argocd_server --insecure --username admin --password $password
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
    local aws=`kubectl create secret generic aws-config --from-literal=AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID:?} --from-literal=AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY:?} --dry-run=client -o yaml`

    local ARGO_TOKEN=`kubectl get secret -n argo default.service-account-token -o=jsonpath='{.data.token}' | base64 --decode`
    local scan_automate_api=`kubectl create secret generic scan-automate-api \
        --from-literal=JWT_SECRET=${JWT_SECRET:?} \
        --from-literal=ARGO_WORKFLOW_TOKEN="Bearer $ARGO_TOKEN" \
        --dry-run=client -o yaml`
    local smtp=`kubectl create secret generic smtp-config \
        --from-literal=SMTP_HOST=${SMTP_HOST:?} \
        --from-literal=SMTP_USERNAME=${SMTP_USERNAME:?} \
        --from-literal=SMTP_PASSWORD=${SMTP_PASSWORD:?} \
        --from-literal=SMTP_FROM=${SMTP_FROM:?} \
        --dry-run=client -o yaml`

    echo "$aws" | kubectl apply -n argo -f -
    echo "$docker" | kubectl apply -n argo -f -
    echo "$git" | kubectl apply -n argo -f -

    echo "$scan_automate_api" | kubectl apply -n scan-automate -f -
    echo "$smtp" | kubectl apply -n scan-automate -f -
}

deploy-argocd
ebort -- login-argocd
add-private-repos
deploy-argocd-apps
ebort -- deploy-secrets