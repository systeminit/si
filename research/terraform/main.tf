terraform {
  required_version = ">= 1.0.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 3.46"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.3"
    }
  }
}

variable "region" {
  default = "us-east-2"
}

variable "cluster_name" {
  default = "democluster"
}

variable "ns" {
  default = "ttfn"
}

variable "app_name" {
  default = "whiskers"
}

variable "docker_org" {
  default = "systeminit"
}

provider "aws" {
  region = var.region
}

data "aws_eks_cluster" "cluster" {
  name = var.cluster_name
}

data "aws_eks_cluster_auth" "cluster" {
  name = var.cluster_name
}

provider "kubernetes" {
  host                   = data.aws_eks_cluster.cluster.endpoint
  cluster_ca_certificate = base64decode(data.aws_eks_cluster.cluster.certificate_authority.0.data)
  exec {
    api_version = "client.authentication.k8s.io/v1alpha1"
    command     = "aws"
    args        = ["eks", "get-token", "--cluster-name", var.cluster_name]
  }
}

resource "kubernetes_namespace" "ns" {
  metadata {
    name = var.ns
  }
}

resource "kubernetes_service" "whiskers" {
  metadata {
    name      = format("%s-service", var.app_name)
    namespace = kubernetes_namespace.ns.metadata.0.name
  }

  spec {
    port {
      port     = 80
      protocol = "TCP"
    }
    selector = {
      app = var.app_name
    }
    type = "LoadBalancer"
  }
}

resource "kubernetes_deployment" "whiskers" {
  metadata {
    name = var.app_name
    labels = {
      app = var.app_name
    }
    namespace = kubernetes_namespace.ns.metadata.0.name
  }

  spec {
    selector {
      match_labels = {
        app = var.app_name
      }
    }

    template {
      metadata {
        labels = {
          app = var.app_name
        }

        namespace = kubernetes_namespace.ns.metadata.0.name
      }

      spec {
        container {
          image             = format("%s/%s", var.docker_org, var.app_name)
          image_pull_policy = "Always"
          name              = var.app_name
          port {
            container_port = 80
            protocol       = "TCP"
          }
        }
      }
    }
  }
}

output "kubernetes_endpoint" {
  value = data.aws_eks_cluster.cluster.endpoint
}

output "whiskers_url" {
  value = format("http://%s", kubernetes_service.whiskers.status.0.load_balancer.0.ingress.0.hostname)
}
