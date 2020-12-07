import {
  PropNumber,
  PropObject,
  PropText,
  PropLink,
} from "../../../components/prelude";

import { valiatePropertyList } from "../../../testValidators";
import { registry, PropLookup } from "../../../registry";

import { kubernetesMetadata } from "../base/metadata";

import { kubernetesServiceBackendPort } from "../base/serviceBackendPort";
import { kubernetesIngressServiceBackend } from "../base/ingressServiceBackend";
import { kubernetesTypedLocalObjectReference } from "../base/typedLocalObjectReference";
import { kubernetesIngressBackend } from "../base/ingressBackend";
import { kubernetesIngressSpec } from "../base/ingressSpec";
import { kubernetesHttpIngressPath } from "../base/httpIngressPath";
import { kubernetesHttpIngressRuleValue } from "../base/httpIngressRuleValue";
import { kubernetesIngressRule } from "../base/ingressRule";
import { kubernetesIngressTls } from "../base/ingressTls";
import { kubernetesLoadBalancerStatus } from "../base/loadBalancerStatus";
import { kubernetesIngressStatus } from "../base/ingressStatus";

import { kubernetesIngress } from "./ingress";

test("create kubernetesIngress entity", done => {
  registry.objects = [];

  // [kubernetesIngress Dependencies]
  // -> kubernetesMetadata
  // -> kubernetesIngressSpec
  //   -> kubernetesIngressBackend
  //     -> kubernetesTypedLocalObjectReference
  //     -> kubernetesIngressServiceBackend
  //       -> kubernetesServiceBackendPort
  //   -> kubernetesIngressRule
  //     -> kubernetesHttpIngressRuleValue
  //       -> kubernetesHttpIngressPath
  //         -> kubernetesIngressBackend
  //           -> kubernetesTypedLocalObjectReference
  //           -> kubernetesIngressServiceBackend
  //   -> kubernetesIngressTls
  // -> kubernetesIngressStatus
  // (order matters!)
  registry.base(kubernetesMetadata);
  registry.base(kubernetesServiceBackendPort);
  registry.base(kubernetesIngressServiceBackend);
  registry.base(kubernetesTypedLocalObjectReference);
  registry.base(kubernetesIngressBackend);
  registry.base(kubernetesIngressSpec);
  registry.base(kubernetesHttpIngressPath);
  registry.base(kubernetesHttpIngressRuleValue);
  registry.base(kubernetesIngressRule);
  registry.base(kubernetesIngressTls);
  registry.base(kubernetesLoadBalancerStatus);
  registry.base(kubernetesIngressStatus);

  registry.componentAndEntity(kubernetesIngress);
  let registryObject: any = registry.get("kubernetesIngress");

  valiatePropertyList(registryObject);

  expect(registryObject.typeName).toBe("kubernetesIngress");
  expect(registryObject.iEntity.uiVisible).toBe(true);
  expect(registryObject.iEntity.uiMenuCategory).toBe("kubernetes");
  expect(registryObject.iEntity.uiMenuDisplayName).toBe("ingress");

  let kubernetesObject = registryObject.properties.getEntry(
    "kubernetesObject",
  ) as PropObject;

  let apiVersion = kubernetesObject.properties.getEntry(
    "apiVersion",
  ) as PropText;
  expect(apiVersion.name).toBe("apiVersion");
  expect(apiVersion.label).toBe("API Version");
  expect(apiVersion.repeated).toBe(false);
  expect(apiVersion.required).toBe(true);

  let kind = kubernetesObject.properties.getEntry("kind") as PropText;
  expect(kind.name).toBe("kind");
  expect(kind.label).toBe("Kind");
  expect(kind.baseDefaultValue).toBe("Ingress");
  expect(kind.repeated).toBe(false);
  expect(kind.required).toBe(true);

  let metadata = kubernetesObject.properties.getEntry("metadata") as PropLink;
  expect(metadata.name).toBe("metadata");
  expect(metadata.label).toBe("Metadata");
  expect(metadata.repeated).toBe(false);
  expect(metadata.lookup?.typeName).toBe("kubernetesMetadata");
  registry.lookupProp({ typeName: "kubernetesMetadata" });

  let spec = kubernetesObject.properties.getEntry("spec") as PropLink;
  expect(spec.name).toBe("spec");
  expect(spec.label).toBe("Ingress Spec");
  expect(spec.repeated).toBe(false);
  expect(spec.lookup?.typeName).toBe("kubernetesIngressSpec");
  registry.lookupProp({ typeName: "kubernetesIngressSpec" });

  let status = kubernetesObject.properties.getEntry("status") as PropLink;
  expect(status.name).toBe("status");
  expect(status.label).toBe("Ingress Status");
  expect(status.repeated).toBe(false);
  expect(status.lookup?.typeName).toBe("kubernetesIngressStatus");
  registry.lookupProp({ typeName: "kubernetesIngressStatus" });

  done();
});
