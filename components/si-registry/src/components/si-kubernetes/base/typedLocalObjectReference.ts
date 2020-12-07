import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface TypedLocalObjectReference {
//   apiGroup: string;
//   kind: string;
//   name: string;
// }

let kubernetesTypedLocalObjectReference = {
  typeName: "kubernetesTypedLocalObjectReference",
  displayTypeName: "Kubernetes Typed Local Object Reference",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addText({
      name: "apiGroup",
      label: "Api Group",
    });

    c.fields.addText({
      name: "kind",
      label: "Kind",
    });

    c.fields.addText({
      name: "name",
      label: "Name",
    });
  },
};

export { kubernetesTypedLocalObjectReference };

registry.base(kubernetesTypedLocalObjectReference);
