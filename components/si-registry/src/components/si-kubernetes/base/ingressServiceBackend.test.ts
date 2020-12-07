import { PropLink, PropText } from "../../../components/prelude";
import { registry } from "../../../registry";

import { kubernetesIngressServiceBackend } from "./ingressServiceBackend";

test("create kubernetesIngressServiceBackend base", done => {
  registry.objects = [];
  registry.base(kubernetesIngressServiceBackend);
  let registryObject = registry.get("kubernetesIngressServiceBackend");
  expect(registryObject.typeName).toBe("kubernetesIngressServiceBackend");

  let name = registryObject.fields.getEntry("name") as PropText;
  expect(name.name).toBe("name");
  expect(name.label).toBe("Name");
  expect(name.repeated).toBe(false);

  let port = registryObject.fields.getEntry("port") as PropLink;
  expect(port.name).toBe("port");
  expect(port.label).toBe("Port");
  expect(port.lookup?.typeName).toBe("kubernetesServiceBackendPort");
  expect(port.repeated).toBe(false);

  done();
});
