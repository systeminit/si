import {
  PropNumber,
  PropObject,
  PropText,
  PropLink,
} from "../../../components/prelude";

import { registry } from "../../../registry";

import { kubernetesCluster } from "./cluster";

test("create kubernetesCluster entity", done => {
  registry.componentAndEntity(kubernetesCluster);

  let registryObject: any = registry.get("kubernetesCluster");
  expect(registryObject.typeName).toBe("kubernetesCluster");

  // let kubernetesObject = registryObject.properties.getEntry(
  //   "kubernetesObject",
  // ) as PropObject;

  // let metadata = kubernetesObject.properties.getEntry("metadata") as PropLink;
  // // console.log(metadata.lookupMyself())
  // expect(metadata.name).toBe("metadata");
  // expect(metadata.label).toBe("Metadata");
  // expect(metadata.repeated).toBe(false);
  // expect(metadata.lookup?.typeName).toBe("kubernetesMetadata");

  done();
});
