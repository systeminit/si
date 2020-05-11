import { PropMethod } from "./attrList";
import { PropLink } from "./prop/link";
import { PropText } from "./prop/text";
import { SystemObject } from "./systemComponent";

import { registry } from "./registry";

test("create component and entity", done => {
  registry.componentAndEntity({
    typeName: "kubernetesDeployment",
    displayTypeName: "Kubernetes Deployment Object",
    siPathName: "si-kubernetes",
    serviceName: "kubernetes",
    options(c) {
      c.constraints.addText({
        name: "mybutt",
        label: "your buytt",
      });
    },
  });
  expect(registry.objects.length).toBe(3);
  done();
});

test("createSystemComponent", done => {
  const c = new SystemObject({
    typeName: "billingAccount",
    displayTypeName: "Billing Account",
    siPathName: "si-account",
    serviceName: "whatwhat",
  });

  c.fields.addText({
    name: "something",
    label: "just to have one",
    options(p: PropText) {
      p.universal = true;
    },
  });

  c.methods.addMethod({
    name: "create",
    label: "Create a new billing account",
    options(p: PropMethod) {
      p.request.properties.addText({
        name: "name",
        label: "Billing Account Name",
      });
      p.reply.properties.addLink({
        name: "billingAccount",
        label: "Billing Account",
        options(p: PropLink) {
          p.lookup = {
            typeName: "billingAccount",
            names: [],
          };
        },
      });
    },
  });
  done();
});
