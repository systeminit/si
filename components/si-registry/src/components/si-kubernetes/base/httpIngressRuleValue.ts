import { PropLink } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface HTTPIngressRuleValue {
//   paths: HTTPIngressPath[]
// }

// Depends on:
// - kubernetesHttpIngressPath

let kubernetesHttpIngressRuleValue = {
  typeName: "kubernetesHttpIngressRuleValue",
  displayTypeName: "Kubernetes HTTP Ingress Rule Value",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addLink({
      name: "paths",
      label: "Paths",
      options(p: PropLink) {
        p.repeated = true;
        p.lookup = {
          typeName: "kubernetesHttpIngressPath",
        };
      },
    });
  },
};
export { kubernetesHttpIngressRuleValue };

registry.base(kubernetesHttpIngressRuleValue);
