import { PropLink } from "../../../components/prelude";
import { registry } from "../../../registry";

import { kubernetesIngressBackend } from "./ingressBackend";

test("create kubernetesIngressBackend base", done => {
  registry.objects = [];
  registry.base(kubernetesIngressBackend);
  let registryObject = registry.get("kubernetesIngressBackend");
  expect(registryObject.typeName).toBe("kubernetesIngressBackend");

  let resource = registryObject.fields.getEntry("resource") as PropLink;
  expect(resource.name).toBe("resource");
  expect(resource.label).toBe("Resource");
  expect(resource.lookup?.typeName).toBe("kubernetesTypedLocalObjectReference");
  expect(resource.repeated).toBe(false);

  let service = registryObject.fields.getEntry("service") as PropLink;
  expect(service.name).toBe("service");
  expect(service.label).toBe("Service");
  expect(service.lookup?.typeName).toBe("kubernetesIngressServiceBackend");
  expect(service.repeated).toBe(false);

  done();
});
