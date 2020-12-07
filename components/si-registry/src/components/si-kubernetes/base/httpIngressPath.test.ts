import { PropLink, PropText } from "../../../components/prelude";

import { Props } from "../../../attrList";
import { registry } from "../../../registry";

import { kubernetesHttpIngressPath } from "./httpIngressPath";

test("create kubernetesHttpIngressPath base", done => {
  registry.objects = [];
  registry.base(kubernetesHttpIngressPath);
  let registryObject = registry.get("kubernetesHttpIngressPath");
  expect(registryObject.typeName).toBe("kubernetesHttpIngressPath");

  let backend = registryObject.fields.getEntry("backend") as PropLink;
  expect(backend.name).toBe("backend");
  expect(backend.label).toBe("Backend");
  expect(backend.repeated).toBe(false);
  expect(backend.lookup?.typeName).toBe("kubernetesIngressBackend");

  let path = registryObject.fields.getEntry("path") as PropText;
  expect(path.name).toBe("path");
  expect(path.label).toBe("Path");
  expect(path.repeated).toBe(false);

  let pathType = registryObject.fields.getEntry("pathType") as PropText;
  expect(pathType.name).toBe("pathType");
  expect(pathType.label).toBe("Path Type");
  expect(path.repeated).toBe(false);

  done();
});
