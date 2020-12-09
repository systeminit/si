import { PropText } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

let kubernetesMetadata = {
  typeName: "kubernetesMetadata",
  displayTypeName: "Kubernetes Meta Data",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    c.fields.addText({
      name: "name",
      label: "Name",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.fields.addText({
      name: "namespace",
      label: "Namespace",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.fields.addMap({
      name: "labels",
      label: "Labels",
      // options(p: PropMap) {
      //   p.repeated = true;
      // }
    });
  },
};

export { kubernetesMetadata };

registry.base(kubernetesMetadata);
