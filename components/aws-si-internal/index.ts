import * as pulumi from "@pulumi/pulumi";
import * as eks from "@pulumi/eks";
import * as k8s from "@pulumi/kubernetes";
import * as aws from "@pulumi/aws";

export const release = process.env.RELEASE ? process.env.RELEASE : "latest";
console.log(`You have release ${release}`);

// Create an EKS cluster with the default configuration.
const cluster = new eks.Cluster("si-internal");

export const clusterName = cluster.eksCluster.name;
export const kubeconfig = cluster.kubeconfig;
export const clusterNodeInstanceRoleName = cluster.instanceRoles.apply(
  roles => roles[0].name
);
export const nodesubnetId = cluster.core.subnetIds;

// Provider for the k8s cluster.
const provider = cluster.provider;

// Namespace
const k8sNamespace = new k8s.core.v1.Namespace("si-internal", {
  metadata: {
    name: "si-internal",
  },
}, { provider });
export const namespace = k8sNamespace.metadata.name;

// Pull the docker hub password
const config = new pulumi.Config();
const imagePullSecret = new k8s.core.v1.Secret(
  "docker-hub",
  {
    type: "kubernetes.io/dockerconfigjson",
    metadata: {
      namespace,
    },
    stringData: {
      ".dockerconfigjson": config
      .requireSecret("docker-hub-token")
      .apply(value => {
        return JSON.stringify({
          auths: {
            "https://index.docker.io/v1/": {
              auth: value
            }
          }
        })
      })
    },
  },
  {
    provider,
  }
);

// Couchbase
const couchbaseService = new k8s.core.v1.Service("couchbase-service", {
  metadata: {
    name: "couchbase-service",
    namespace,
    labels: {
      service: "couchbase-service",
      infra: "true",
    },
  },
  spec: {
    selector: {
      app: "couchbase"
    },
    ports: [
      {
        name: "cb1",
        port: 8091,
      },
      {
        name: "cb2",
        port: 8092,
      },
      {
        name: "cb3",
        port: 8093,
      },
      {
        name: "cb4",
        port: 8094,
      },
      {
        name: "cb5",
        port: 8095,
      },
      {
        name: "cb6",
        port: 8096,
      },
      {
        name: "cb7",
        port: 11210,
      },
      {
        name: "cb8",
        port: 11211,
      },
    ],
  }
}, { provider });

const couchbaseDeployment = new k8s.apps.v1.Deployment("couchbase", {
  metadata: {
    name: "couchbase",
    namespace,
    labels: {
      app: "couchbase",
      infra: "true",
    },
  },
  spec: {
    replicas: 1,
    selector: {
      matchLabels: {
        app: "couchbase",
      },
    },
    template: {
      metadata: {
        labels: {
          app: "couchbase",
          infra: "true",
        }
      },
      spec: {
        imagePullSecrets: [{ name: imagePullSecret.metadata.apply(m => m.name) }],
        containers: [
          {
            image: "systeminit/couchbase:latest",
            name: "couchbase",
            ports: [
              {
                containerPort: 8091,
              },
              {
                containerPort: 8092,
              },
              {
                containerPort: 8093,
              },
              {
                containerPort: 8094,
              },
              {
                containerPort: 8095,
              },
              {
                containerPort: 8096,
              },
              {
                containerPort: 11210,
              },
              {
                containerPort: 11211,
              },
            ],
          }
        ]
      }
    },
  }
}, { provider });

// NATS
const natsService = new k8s.core.v1.Service("nats-service", {
  metadata: {
    name: "nats-service",
    namespace,
    labels: {
      service: "nats-service",
      infra: "true",
    },
  },
  spec: {
    selector: {
      app: "nats"
    },
    ports: [
      {
        name: "nats",
        port: 4222,
      },
    ],
  }
}, { provider });

const natsDeployment = new k8s.apps.v1.Deployment("nats", {
  metadata: {
    name: "nats",
    namespace,
    labels: {
      app: "nats",
      infra: "true",
    },
  },
  spec: {
    replicas: 1,
    selector: {
      matchLabels: {
        app: "nats",
      },
    },
    template: {
      metadata: {
        labels: {
          app: "nats",
          infra: "true",
        }
      },
      spec: {
        imagePullSecrets: [{ name: imagePullSecret.metadata.apply(m => m.name) }],
        containers: [
          {
            image: "systeminit/nats:latest",
            name: "nats",
            ports: [
              {
                containerPort: 4222,
              },
            ],
          }
        ]
      }
    },
  }
}, { provider });

// si-sdf
const sdfService = new k8s.core.v1.Service("si-sdf-service", {
  metadata: {
    name: "si-sdf-service",
    namespace,
    labels: {
      service: "si-sdf-service",
      infra: "true",
    },
  },
  spec: {
    selector: {
      app: "si-sdf"
    },
    ports: [
      {
        name: "si-sdf",
        port: 5156,
      },
    ],
    type: "NodePort",
  }
}, { provider });

const sdfConfig = new k8s.core.v1.Secret("si-sdf-config", {
  metadata: {
    namespace,
    name: "si-sdf-config",
  },
  stringData: {
    "default.toml": config.requireSecret("si-sdf-config").apply(s => s),
    "private.pem": config.requireSecret("jwt-private").apply(s => s),
    "public.pem": config.require("jwt-public"),
  },
}, { provider });

const sdfDeployment = new k8s.apps.v1.Deployment("si-sdf", {
  metadata: {
    name: "si-sdf",
    namespace,
    labels: {
      app: "si-sdf",
      infra: "true",
    },
  },
  spec: {
    replicas: 1,
    selector: {
      matchLabels: {
        app: "si-sdf",
      },
    },
    template: {
      metadata: {
        labels: {
          app: "si-sdf",
          infra: "true",
        }
      },
      spec: {
        imagePullSecrets: [{ name: imagePullSecret.metadata.apply(m => m.name) }],
        containers: [
          {
            image: `systeminit/si-sdf:${release}`,
            name: "si-sdf",
            ports: [
              {
                containerPort: 5156,
              },
            ],
            volumeMounts: [
              {
                name: "si-sdf-config-volume",
                mountPath: "/svc/si-sdf/config",
                readOnly: true,
              }
            ],
            env: [
              { 
                name: "RUST_LOG",
                value: "debug",
              },
            ],
          },
          {
            image: `systeminit/si-veritech:${release}`,
            name: "si-veritech",
            ports: [
              {
                containerPort: 5157,
              },
            ],
          }
        ],
        volumes: [
          {
            name: "si-sdf-config-volume",
            secret: {
              secretName: "si-sdf-config",
            }
          }
        ]
      }
    },
  }
}, { provider });

// Web App 
const webAppService = new k8s.core.v1.Service("si-web-app-service", {
  metadata: {
    name: "si-web-app-service",
    namespace,
    labels: {
      service: "si-web-app-service",
      infra: "true",
    },
  },
  spec: {
    selector: {
      app: "si-web-app"
    },
    ports: [
      {
        name: "si-web-app",
        port: 80,
      },
    ],
    type: "NodePort",
  }
}, { provider });


const siWebAppDeployment = new k8s.apps.v1.Deployment("si-web-app", {
  metadata: {
    name: "si-web-app",
    namespace,
    labels: {
      app: "si-web-app",
      infra: "true",
    },
  },
  spec: {
    replicas: 1,
    selector: {
      matchLabels: {
        app: "si-web-app",
      },
    },
    template: {
      metadata: {
        labels: {
          app: "si-web-app",
          infra: "true",
        }
      },
      spec: {
        imagePullSecrets: [{ name: imagePullSecret.metadata.apply(m => m.name) }],
        containers: [
          {
            image: `systeminit/si-web-app:${release}`,
            name: "si-web-app",
            ports: [
              {
                containerPort: 80,
              },
            ],
            env: [
              { 
                name: "VUE_APP_SDF",
                value: "https://internal-api.systeminit.com",
              },
              { 
                name: "VUE_APP_SDF_WS",
                value: "wss://internal-api.systeminit.com/updates",
              },
            ],
          },
        ],
      }
    },
  }
}, { provider });

// Application Load Balancer ingress controller

// Create IAM Policy for the IngressController called "ingressController-iam-policy” and read the policy ARN.
const ingressControllerPolicy = new aws.iam.Policy(
  "ingressController-iam-policy",
  {
    policy: {
      "Version": "2012-10-17",
      "Statement": [
        {
          "Effect": "Allow",
          "Action": [
            "acm:DescribeCertificate",
            "acm:ListCertificates",
            "acm:GetCertificate"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "ec2:AuthorizeSecurityGroupIngress",
            "ec2:CreateSecurityGroup",
            "ec2:CreateTags",
            "ec2:DeleteTags",
            "ec2:DeleteSecurityGroup",
            "ec2:DescribeAccountAttributes",
            "ec2:DescribeAddresses",
            "ec2:DescribeInstances",
            "ec2:DescribeInstanceStatus",
            "ec2:DescribeInternetGateways",
            "ec2:DescribeNetworkInterfaces",
            "ec2:DescribeSecurityGroups",
            "ec2:DescribeSubnets",
            "ec2:DescribeTags",
            "ec2:DescribeVpcs",
            "ec2:ModifyInstanceAttribute",
            "ec2:ModifyNetworkInterfaceAttribute",
            "ec2:RevokeSecurityGroupIngress"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "elasticloadbalancing:AddListenerCertificates",
            "elasticloadbalancing:AddTags",
            "elasticloadbalancing:CreateListener",
            "elasticloadbalancing:CreateLoadBalancer",
            "elasticloadbalancing:CreateRule",
            "elasticloadbalancing:CreateTargetGroup",
            "elasticloadbalancing:DeleteListener",
            "elasticloadbalancing:DeleteLoadBalancer",
            "elasticloadbalancing:DeleteRule",
            "elasticloadbalancing:DeleteTargetGroup",
            "elasticloadbalancing:DeregisterTargets",
            "elasticloadbalancing:DescribeListenerCertificates",
            "elasticloadbalancing:DescribeListeners",
            "elasticloadbalancing:DescribeLoadBalancers",
            "elasticloadbalancing:DescribeLoadBalancerAttributes",
            "elasticloadbalancing:DescribeRules",
            "elasticloadbalancing:DescribeSSLPolicies",
            "elasticloadbalancing:DescribeTags",
            "elasticloadbalancing:DescribeTargetGroups",
            "elasticloadbalancing:DescribeTargetGroupAttributes",
            "elasticloadbalancing:DescribeTargetHealth",
            "elasticloadbalancing:ModifyListener",
            "elasticloadbalancing:ModifyLoadBalancerAttributes",
            "elasticloadbalancing:ModifyRule",
            "elasticloadbalancing:ModifyTargetGroup",
            "elasticloadbalancing:ModifyTargetGroupAttributes",
            "elasticloadbalancing:RegisterTargets",
            "elasticloadbalancing:RemoveListenerCertificates",
            "elasticloadbalancing:RemoveTags",
            "elasticloadbalancing:SetIpAddressType",
            "elasticloadbalancing:SetSecurityGroups",
            "elasticloadbalancing:SetSubnets",
            "elasticloadbalancing:SetWebACL"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "iam:CreateServiceLinkedRole",
            "iam:GetServerCertificate",
            "iam:ListServerCertificates"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "cognito-idp:DescribeUserPoolClient"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "waf-regional:GetWebACLForResource",
            "waf-regional:GetWebACL",
            "waf-regional:AssociateWebACL",
            "waf-regional:DisassociateWebACL"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "tag:GetResources",
            "tag:TagResources"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "waf:GetWebACL"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "wafv2:GetWebACL",
            "wafv2:GetWebACLForResource",
            "wafv2:AssociateWebACL",
            "wafv2:DisassociateWebACL"
          ],
          "Resource": "*"
        },
        {
          "Effect": "Allow",
          "Action": [
            "shield:DescribeProtection",
            "shield:GetSubscriptionState",
            "shield:DeleteProtection",
            "shield:CreateProtection",
            "shield:DescribeSubscription",
            "shield:ListProtections"
          ],
          "Resource": "*"
        }
      ]
    }
  }
);

// Attach this policy to the NodeInstanceRole of the worker nodes.
export const nodeinstanceRole = new aws.iam.RolePolicyAttachment(
  "eks-NodeInstanceRole-policy-attach",
  {
    policyArn: ingressControllerPolicy.arn,
    role: clusterNodeInstanceRoleName
  }
);

// Declare the ALBIngressController in 1 step with the Helm Chart.
const albingresscntlr = new k8s.helm.v3.Chart(
  "alb",
  {
    chart: "aws-alb-ingress-controller",
    fetchOpts: {
      repo: 'http://storage.googleapis.com/kubernetes-charts-incubator',
    },
    values: {
      clusterName: clusterName,
      autoDiscoverAwsRegion: "true",
      autoDiscoverAwsVpcID: "true",
      namespace,
    }
  },
  { provider },
);

export const ingressApi = new k8s.extensions.v1beta1.Ingress("internal-api", {
  metadata: {
    name: "internal-api-ingress",
    namespace,
    annotations: {
      "kubernetes.io/ingress.class": "alb",
      "alb.ingress.kubernetes.io/scheme": "internet-facing",
      "alb.ingress.kubernetes.io/certificate-arn": "arn:aws:acm:us-east-2:835304779882:certificate/75582463-9615-40b1-9d49-cbde7cf5abb3",
      "alb.ingress.kubernetes.io/listen-ports": '[{"HTTP": 80}, {"HTTPS":443}]',
      "alb.ingress.kubernetes.io/actions.ssl-redirect": '{"Type": "redirect", "RedirectConfig": { "Protocol": "HTTPS", "Port": "443", "StatusCode": "HTTP_301"}}',
    },
    labels: {
      app: "internal-api-ingress",
    },
  },
  spec: {
    tls: [
      {
        hosts: [ "internal-api.systeminit.com" ],
      },
    ],
    rules: [
      { 
        http: {
          paths: [
            //{ 
            //  path: "/*",
            //  backend: {
            //    serviceName: "ssl-redirect",
            //    servicePort: "use-annotation",
            //  },
            //},
            { 
              path: "/*",
              backend: {
                serviceName: "si-sdf-service",
                servicePort: 5156,
              },
            },
          ],
        },
      },
    ],
  }
}, { provider });

export const ingressInternalWeb = new k8s.extensions.v1beta1.Ingress("internal", {
  metadata: {
    name: "internal-ingress",
    namespace,
    annotations: {
      "kubernetes.io/ingress.class": "alb",
      "alb.ingress.kubernetes.io/scheme": "internet-facing",
      "alb.ingress.kubernetes.io/certificate-arn": 
"arn:aws:acm:us-east-2:835304779882:certificate/a84e9383-e769-46b6-9834-7758f254cf99",
      "alb.ingress.kubernetes.io/listen-ports": '[{"HTTP": 80}, {"HTTPS":443}]',
      "alb.ingress.kubernetes.io/actions.ssl-redirect": '{"Type": "redirect", "RedirectConfig": { "Protocol": "HTTPS", "Port": "443", "StatusCode": "HTTP_301"}}',
    },
    labels: {
      app: "internal-api-ingress",
    },
  },
  spec: {
    tls: [
      {
        hosts: [ "internal-api.systeminit.com" ],
      },
    ],
    rules: [
      { 
        http: {
          paths: [
            //{ 
            //  path: "/*",
            //  backend: {
            //    serviceName: "ssl-redirect",
            //    servicePort: "use-annotation",
            //  },
            //},
            { 
              path: "/*",
              backend: {
                serviceName: "si-web-app-service",
                servicePort: 80,
              },
            },
          ],
        },
      },
    ],
  }
}, { provider });


