import { PropLink } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface IngressStatus {
//   loadBalancer: LoadBalancerStatus;
// }

// Depends on:
// - kubernetesLoadBalancerStatus

let kubernetesIngressStatus = {
  typeName: "kubernetesIngressStatus",
  displayTypeName: "Kubernetes Ingress Status",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addLink({
      name: "LoadBalancerStatus",
      label: "Load Balancer Status",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesLoadBalancerStatus",
        };
      },
    });
  },
};

export { kubernetesIngressStatus };

registry.base(kubernetesIngressStatus);
