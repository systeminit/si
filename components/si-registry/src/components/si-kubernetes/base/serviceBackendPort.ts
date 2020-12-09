import { PropNumber } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface ServiceBackendPort {
//   name: string;
//   number: number; //int
// }

let kubernetesServiceBackendPort = {
  typeName: "kubernetesServiceBackendPort",
  displayTypeName: "Kubernetes Service Backend Port",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addText({
      name: "name",
      label: "Name",
    });

    c.fields.addNumber({
      name: "number",
      label: "Number",
      options(p: PropNumber) {
        p.numberKind = "int32";
      },
    });
  },
};

export { kubernetesServiceBackendPort };

registry.base(kubernetesServiceBackendPort);
