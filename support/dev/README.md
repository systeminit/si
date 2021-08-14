# Deploying systeminit

1. Deploy systeminit to minikube
1. Compose systeminit with systeminit (minikube)
1. Deploy systeminit to eks
1. Compose whiskers with systeminit (eks)
1. Deploy whiskers to eks  

# Deploy systeminit to minikube

## Dependencies

* Minikube
* Docker _(if not running minikube in vm)_
* kubectl _(make sure the version is compatible with the k8s version running on minukube)_

## Preflight

### Docker Images

#### Build images

Build the images with:

```./demo-build..sh```

Or build a specific image with:

```./demo-build.sh veritech```

#### Tag images

Tag each images with:
```docker tag dev_veritech:latest "systeminit/si-veritech:$(date -u +%Y%m%d.%H%M%S).0-sha.$(git show -s --format=%h)"```

#### Push images

Push images with:
```docker image push systeminit/si-veritech:_the_image_tag_from_above_```


#### Updating the image in kubernetes deployments 
Update the kubernetes deployment.container.image for the components that have a new image...

* sdf-deployment.yaml
* veritech-deployment.yaml
* web-deployment.yaml

### Configuration

#### Dockerhub configuration
* Create dockerconfig.json from k8s/base/dockerconfig.json.sample

```cp k8s/base/dockerconfig.json.sample k8s/base/dockerconfig.json```

* generate base64 auth (the password should be an auth token)

```openssl base64 username:password```

* set the auth value _(in dockerconfig.json)_ to the base64 encoded string

#### Honeycomb configuration
* Create honeycomb.env from k8s/base/honeycomb.env.sample

```cp k8s/base/honeycomb.env.sample k8s/base/honeycomb.env```

* Set dataset and token fields in _(k8s/base/honeycomb.env)_

#### Database configuration
* Create postgres.env from k8s/default/postgres.env

```cp k8s/default/postgres.env.sample k8s/default/postgres.env```

* Set db, username, and password fields in _(k8s/default/postgres.env)_

#### Auth to access app.systeminit.com 
* create an htpasswd file
htpasswd -c k8s/default/web-htpasswd  sidev

## Launching Minikube

#### Minikube commands reference

```
minikube start   # start a new cluster
minikube stop    # stop the cluster
minikube delete  # delete the cluster
```

[Minikube commands documentation](https://minikube.sigs.k8s.io/docs/commands/)

### Launch minikube
Make sure the minikube cluster is running the same version as the eks cluster.   

```minikube start --kubernetes-version=v1.19.6```

### Enable minikube ingress
```minikube addons enable ingress```

If having issues enabling ingress, delete the minikube cluster and relaunch with

```minikube start --kubernetes-version=v1.19.6 --vm=true```

## Deploying systeminit to minikube
* Create a namespace (create a new one as needed)

```kubectl apply -f k8s/namespace/sidvlp-namespace.yaml```

* Deploy SI

```kubectl apply --namespace sidvlp -k k8s/default/```

## Accessing systeminit on minikube

* Get the ip address of the minikube cluster

```minikube ip```

* Create an entry in your hosts file _(/etc/hosts)_

```[ip of the minikube cluster] app.systeminit.com```

Signup

* [http://app.systeminit.com/authenticate/signup](http://app.systeminit.com/authenticate/signup)


