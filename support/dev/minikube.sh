#!/usr/bin/env sh


# Start minikube
minikube start --kubernetes-version=v1.19.6 --addons=ingress --vm=true

# Deploy namespace
kubectl apply -f k8s/namespace/sidvlp-namespace.yaml

# Deploy systeminit with local postgres
kubectl apply --namespace sidvlp -k k8s/default

# Get minikube ip
minikube ip
