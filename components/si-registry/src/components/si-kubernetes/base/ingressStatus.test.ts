import { PropLink, PropText } from "../../../components/prelude";
import { registry } from "../../../registry";

import { kubernetesIngressStatus } from "./ingressStatus";

test("create kubernetesIngressStatus base", done => {
  registry.objects = [];
  registry.base(kubernetesIngressStatus);
  let registryObject = registry.get("kubernetesIngressStatus");
  expect(registryObject.typeName).toBe("kubernetesIngressStatus");

  let loadBalancer = registryObject.fields.getEntry(
    "LoadBalancerStatus",
  ) as PropLink;
  expect(loadBalancer.name).toBe("LoadBalancerStatus");
  expect(loadBalancer.label).toBe("Load Balancer Status");
  expect(loadBalancer.lookup?.typeName).toBe("kubernetesLoadBalancerStatus");
  expect(loadBalancer.repeated).toBe(false);

  done();
});
