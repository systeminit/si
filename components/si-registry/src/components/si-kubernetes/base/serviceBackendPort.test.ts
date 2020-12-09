import { PropNumber, PropText } from "../../../components/prelude";

import { registry } from "../../../registry";

import { kubernetesServiceBackendPort } from "./serviceBackendPort";

test("create kubernetesIngressRule base", done => {
  registry.objects = [];
  registry.base(kubernetesServiceBackendPort);
  let registryObject = registry.get("kubernetesServiceBackendPort");
  expect(registryObject.typeName).toBe("kubernetesServiceBackendPort");

  let name = registryObject.fields.getEntry("name") as PropText;
  expect(name.name).toBe("name");
  expect(name.label).toBe("Name");
  expect(name.repeated).toBe(false);

  let number = registryObject.fields.getEntry("number") as PropNumber;
  expect(number.name).toBe("number");
  expect(number.label).toBe("Number");
  expect(number.repeated).toBe(false);
  expect(number.numberKind).toBe("int32");

  done();
});
