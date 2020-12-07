import { PropLink } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface IngressRule {
//   host: string;
//   http: HTTPIngressRuleValue;
// }

// Depends on:
// - kubernetesHttpIngressRuleValue

let kubernetesIngressRule = {
  typeName: "kubernetesIngressRule",
  displayTypeName: "Kubernetes Ingress Rule",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addText({
      name: "host",
      label: "Host",
    });

    c.fields.addLink({
      name: "http",
      label: "Http",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesHttpIngressRuleValue",
        };
      },
    });
  },
};

export { kubernetesIngressRule };

registry.base(kubernetesIngressRule);
