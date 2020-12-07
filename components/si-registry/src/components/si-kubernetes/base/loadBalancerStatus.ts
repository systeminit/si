import { PropObject } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface LoadBalancerStatus {
//   ingress: LoadBalancerIngress [];
// }

// Depends on:
// - kubernetesLoadBalancerIngress __need to implement__

let kubernetesLoadBalancerStatus = {
  typeName: "kubernetesLoadBalancerStatus",
  displayTypeName: "Kubernetes Load Balancer Status",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    c.fields.addObject({
      name: "ingress",
      label: "Load Balancer Status",
      options(p: PropObject) {
        p.repeated = true;
        p.properties.addText({
          name: "hostname",
          label: "Hostname",
        });
        p.properties.addText({
          name: "ip",
          label: "IP",
        });
      },
    });
  },
};

export { kubernetesLoadBalancerStatus };
registry.base(kubernetesLoadBalancerStatus);
