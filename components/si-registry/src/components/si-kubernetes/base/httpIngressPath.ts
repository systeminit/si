import { PropLink } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface HTTPIngressPath {
//   backend: IngressBackend;
//   path: string;
//   pathType: string;
// }

// Depends on:
// - kubernetesIngressBackend

let kubernetesHttpIngressPath = {
  typeName: "kubernetesHttpIngressPath",
  displayTypeName: "Kubernetes HTTP Ingress Path",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addLink({
      name: "backend",
      label: "Backend",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesIngressBackend",
        };
      },
    });
    c.fields.addText({
      name: "path",
      label: "Path",
    });
    c.fields.addText({
      name: "pathType",
      label: "Path Type",
    });
  },
};
export { kubernetesHttpIngressPath };

registry.base(kubernetesHttpIngressPath);
