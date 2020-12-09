import { PropLink } from "../../../components/prelude";
import { registry } from "../../../registry";

import { kubernetesHttpIngressRuleValue } from "./httpIngressRuleValue";

test("create kubernetesHttpIngressRuleValue base", done => {
  registry.objects = [];
  registry.base(kubernetesHttpIngressRuleValue);
  let registryObject = registry.get("kubernetesHttpIngressRuleValue");
  expect(registryObject.typeName).toBe("kubernetesHttpIngressRuleValue");

  let paths = registryObject.fields.getEntry("paths") as PropLink;
  expect(paths.name).toBe("paths");
  expect(paths.label).toBe("Paths");
  expect(paths.lookup?.typeName).toBe("kubernetesHttpIngressPath");
  expect(paths.repeated).toBe(true);

  done();
});
