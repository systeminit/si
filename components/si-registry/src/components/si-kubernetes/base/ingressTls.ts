import { PropText } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface IngressTLS {
//   hosts: string[];
//   secretName: string;
// }

let kubernetesIngressTls = {
  typeName: "kubernetesIngressTls",
  displayTypeName: "Kubernetes Ingress TLS",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addText({
      name: "hosts",
      label: "Hosts",
      options(p: PropText) {
        p.repeated = true;
      },
    });

    c.fields.addText({
      name: "secretName",
      label: "Secret Name",
    });
  },
};

export { kubernetesIngressTls };

registry.base(kubernetesIngressTls);
