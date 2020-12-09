import { PropLink, PropText } from "../../../components/prelude";
import { registry } from "../../../registry";

import { kubernetesIngressSpec } from "./ingressSpec";

test("create kubernetesIngressSpec base", done => {
  registry.objects = [];
  registry.base(kubernetesIngressSpec);
  let registryObject = registry.get("kubernetesIngressSpec");
  expect(registryObject.typeName).toBe("kubernetesIngressSpec");

  let ingressBackend = registryObject.fields.getEntry(
    "ingressBackend",
  ) as PropLink;
  expect(ingressBackend.name).toBe("ingressBackend");
  expect(ingressBackend.label).toBe("Ingress Backend");
  expect(ingressBackend.lookup?.typeName).toBe("kubernetesIngressBackend");
  expect(ingressBackend.repeated).toBe(false);

  let ingressClassName = registryObject.fields.getEntry(
    "ingressClassName",
  ) as PropText;
  expect(ingressClassName.name).toBe("ingressClassName");
  expect(ingressClassName.label).toBe("Ingress Class Name");
  expect(ingressClassName.repeated).toBe(false);

  let ingressRule = registryObject.fields.getEntry("ingressRule") as PropLink;
  expect(ingressRule.name).toBe("ingressRule");
  expect(ingressRule.label).toBe("Ingress Rule");
  expect(ingressRule.lookup?.typeName).toBe("kubernetesIngressRule");
  expect(ingressRule.repeated).toBe(true);

  let ingressTls = registryObject.fields.getEntry("ingressTls") as PropLink;
  expect(ingressTls.name).toBe("ingressTls");
  expect(ingressTls.label).toBe("Ingress TLS");
  expect(ingressTls.lookup?.typeName).toBe("kubernetesIngressTls");
  expect(ingressTls.repeated).toBe(true);

  done();
});
