import { PropLink } from "../../../components/prelude";

import { registry } from "../../../registry";

registry.base({
  typeName: "kubernetesContainer",
  displayTypeName: "Kubernetes Container Definition",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addText({
      name: "name",
      label: "Name",
    });
    c.fields.addText({
      name: "image",
      label: "Image",
    });
    c.fields.addLink({
      name: "ports",
      label: "Ports",
      options(p: PropLink) {
        p.repeated = true;
        p.lookup = {
          typeName: "kubernetesContainerPort",
        };
      },
    });
  },
});
