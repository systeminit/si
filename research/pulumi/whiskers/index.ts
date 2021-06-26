import * as k8s from "@pulumi/kubernetes";
import * as kx from "@pulumi/kubernetesx";

const name = "whiskers";
const ns = "pullmeup";
const appLabels = { app: name };
const image = `systeminit/${name}:latest`;
const port = 80;
const protocol = "TCP";

const namespace = new k8s.core.v1.Namespace(ns);

const service = new k8s.core.v1.Service(`${name}-service`, {
  metadata: {
    namespace: namespace.metadata.name,
  },
  spec: {
    selector: appLabels,
    type: "LoadBalancer",
    ports: [{ port, protocol }],
  },
});

const deployment = new k8s.apps.v1.Deployment(`${name}-deployment`, {
  metadata: {
    labels: appLabels,
    namespace: namespace.metadata.name,
  },
  spec: {
    selector: { matchLabels: appLabels },
    template: {
      metadata: { labels: appLabels },
      spec: {
        containers: [
          {
            name,
            image,
            imagePullPolicy: "Always",
            ports: [{ containerPort: port, protocol }],
          },
        ],
      },
    },
  },
});

export const whiskersUrl = service.status.loadBalancer.apply(
  (lb) => `http://${lb.ingress[0].hostname}`
);
