import { PropText } from "../../../components/prelude";
import { registry } from "../../../registry";

import { kubernetesTypedLocalObjectReference } from "./typedLocalObjectReference";

test("create kubernetesTypedLocalObjectReference base", done => {
  registry.objects = [];
  registry.base(kubernetesTypedLocalObjectReference);
  let registryObject = registry.get("kubernetesTypedLocalObjectReference");
  expect(registryObject.typeName).toBe("kubernetesTypedLocalObjectReference");

  let apiGroup = registryObject.fields.getEntry("apiGroup") as PropText;
  expect(apiGroup.name).toBe("apiGroup");
  expect(apiGroup.label).toBe("Api Group");
  expect(apiGroup.repeated).toBe(false);

  let kind = registryObject.fields.getEntry("kind") as PropText;
  expect(kind.name).toBe("kind");
  expect(kind.label).toBe("Kind");
  expect(kind.repeated).toBe(false);

  let name = registryObject.fields.getEntry("name") as PropText;
  expect(name.name).toBe("name");
  expect(name.label).toBe("Name");
  expect(name.repeated).toBe(false);

  done();
});
