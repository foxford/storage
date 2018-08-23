#!/usr/bin/env bash

export NAMESPACE=$(if [[ ${TRAVIS_TAG} ]]; then echo 'production'; else echo 'staging'; fi)
export DOCKER_IMAGE_TAG=$(if [[ ${TRAVIS_TAG} ]]; then echo ${TRAVIS_TAG}; else echo $(git rev-parse --short HEAD); fi)

set -ex

mkdir -p ${HOME}/.local/bin
export PATH=${HOME}/.local/bin:${PATH}

curl -fsSLo kubectl "https://storage.googleapis.com/kubernetes-release/release/$(curl -s https://storage.googleapis.com/kubernetes-release/release/stable.txt)/bin/linux/amd64/kubectl" \
    && chmod +x kubectl \
    && mv kubectl "${HOME}/.local/bin"
curl -fsSLo skaffold "https://storage.googleapis.com/skaffold/releases/latest/skaffold-linux-amd64" \
    && chmod +x skaffold \
    && mv skaffold "${HOME}/.local/bin"

kubectl config set-cluster media --embed-certs --server ${KUBE_SERVER} --certificate-authority deploy/ca.crt
kubectl config set-credentials travis --token ${KUBE_TOKEN}
kubectl config set-context media --cluster media --user travis --namespace=${NAMESPACE}
kubectl config use-context media

curl -fsSL \
    --header "authorization: token ${GITHUB_TOKEN}" \
    --header "accept: application/vnd.github.v3.raw" \
    "https://api.github.com/repos/netology-group/environment/contents/cluster/k8s/apps/storage/ns/${NAMESPACE}/storage-environment.yaml" \
    | kubectl apply -f -

echo ${DOCKER_PASSWORD} | docker login -u ${DOCKER_USERNAME} --password-stdin

skaffold run
