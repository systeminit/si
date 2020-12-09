import { PropLink, PropText } from "../../../components/prelude";

import { registry } from "../../../registry";

import { kubernetesIngressTls } from "./ingressTls";

test("create kubernetesIngressTls base", done => {
  registry.objects = [];
  registry.base(kubernetesIngressTls);
  let registryObject = registry.get("kubernetesIngressTls");
  expect(registryObject.typeName).toBe("kubernetesIngressTls");

  let hosts = registryObject.fields.getEntry("hosts") as PropText;
  expect(hosts.name).toBe("hosts");
  expect(hosts.label).toBe("Hosts");
  expect(hosts.repeated).toBe(true);

  let secretName = registryObject.fields.getEntry("secretName") as PropText;
  expect(secretName.name).toBe("secretName");
  expect(secretName.label).toBe("Secret Name");
  expect(secretName.repeated).toBe(false);

  done();
});
