import { PropLink } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface IngressSpec {
//   defaultBackend: IngressBackend;
//   ingressClassName: string;
//   rules: IngressRule[];
//   tls: IngressTLS[];
// }

// Depends on:
// - kubernetesIngressBackend
// - kubernetesIngressRule
// - kubernetesIngressTls

let kubernetesIngressSpec = {
  typeName: "kubernetesIngressSpec",
  displayTypeName: "Kubernetes Ingress Spec",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    c.fields.addLink({
      name: "ingressBackend",
      label: "Ingress Backend",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesIngressBackend",
        };
      },
    });

    c.fields.addText({
      name: "ingressClassName",
      label: "Ingress Class Name",
    });

    c.fields.addLink({
      name: "ingressRule",
      label: "Ingress Rule",
      options(p: PropLink) {
        p.repeated = true;
        p.lookup = {
          typeName: "kubernetesIngressRule",
        };
      },
    });

    c.fields.addLink({
      name: "ingressTls",
      label: "Ingress TLS",
      options(p: PropLink) {
        p.repeated = true;
        p.lookup = {
          typeName: "kubernetesIngressTls",
        };
      },
    });
  },
};

export { kubernetesIngressSpec };

registry.base(kubernetesIngressSpec);
