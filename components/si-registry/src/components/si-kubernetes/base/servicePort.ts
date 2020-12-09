import { PropText, PropNumber } from "../../../components/prelude";

import { registry } from "../../../registry";

registry.base({
  typeName: "kubernetesContainerPort",
  displayTypeName: "Kubernetes Container Port Definition",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addNumber({
      name: "containerPort",
      label: "Container Port",
      options(p: PropNumber) {
        p.numberKind = "int32";
      },
    });
    c.fields.addText({
      name: "hostIp", // disabled auto/camelcase in graphql.ts for testing ...
      // name: "hostIP",
      label: "Host IP",
      options(p: PropText) {
        p.hidden = true;
      },
    });
    // temporary commenting out because it must be a number not a string.
    // need to clear empty fields before submitting..
    // c.fields.addNumber({
    //   name: "hostPort",
    //   label: "Host Port",
    //   options(p: PropNumber) {
    //     p.numberKind = "int32";
    //     p.hidden = false;
    //   },
    // });
    c.fields.addText({
      name: "name",
      label: "Name",
      options(p: PropText) {
        p.hidden = true;
      },
    });
    c.fields.addText({
      name: "protocol",
      label: "Protocol",
    });
  },
});
