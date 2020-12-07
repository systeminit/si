import { PropLink, PropText } from "../../../components/prelude";
import { registry } from "../../../registry";

import { kubernetesIngressRule } from "./ingressRule";

test("create kubernetesIngressRule base", done => {
  registry.objects = [];
  registry.base(kubernetesIngressRule);
  let registryObject = registry.get("kubernetesIngressRule");
  expect(registryObject.typeName).toBe("kubernetesIngressRule");

  let host = registryObject.fields.getEntry("host") as PropText;
  expect(host.name).toBe("host");
  expect(host.label).toBe("Host");
  expect(host.repeated).toBe(false);

  let http = registryObject.fields.getEntry("http") as PropLink;
  expect(http.name).toBe("http");
  expect(http.label).toBe("Http");
  expect(http.lookup?.typeName).toBe("kubernetesHttpIngressRuleValue");
  expect(http.repeated).toBe(false);

  done();
});
