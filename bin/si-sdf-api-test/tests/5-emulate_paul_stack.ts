// deno-lint-ignore-file no-explicit-any
import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
  runWithTemporaryChangeset,
  sleep,
  sleepBetween,
  createComponent,
  getPropertyEditor,
  setAttributeValue,
  attributeValueIdForPropPath,
  getQualificationSummary,
  getActions,
  getFuncs,
  extractSchemaVariant,
  getSchemaVariants,
} from "../test_helpers.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";

export default async function emulate_paul_stack(sdfApiClient: SdfApiClient) {
  await sleepBetween(0, 750);
  return runWithTemporaryChangeset(sdfApiClient, emulate_paul_stack_inner);
}

async function emulate_paul_stack_inner(
  sdf: SdfApiClient,
  changeSetId: string,
) {
  sdf.listenForDVUs();
  // LOAD INITIAL DATA
  const { schemaVariants, newCreateComponentApi } = await getSchemaVariants(
    sdf,
    changeSetId,
  );

  const awsRegionVariant = await extractSchemaVariant(
    schemaVariants,
    "Region",
    "AWS",
  );
  const awsRegionVariantId = awsRegionVariant.schemaVariantId;

  const _diagram = await getDiagram(sdf, changeSetId);
  //await sleepBetween(2000, 10000);

  // CREATE COMPONENTS
  // create region component
  const regionComponentId = await createComponent(
    sdf,
    changeSetId,
    awsRegionVariantId,
    0,
    0,
    undefined,
    newCreateComponentApi,
  );

  //await sleepBetween(3000, 6000);

  // UPDATE REGION
  const regionValue = "us-east-1";
  const { values: RegionPropValues } = await getPropertyEditor(
    sdf,
    changeSetId,
    regionComponentId,
  );
  // await sleep(2000);

  {
    const { attributeValueId, parentAttributeValueId, propId } =
      attributeValueIdForPropPath(
        "/root/domain/region",
        awsRegionVariant.props,
        RegionPropValues,
      );

    await setAttributeValue(
      sdf,
      changeSetId,
      regionComponentId,
      attributeValueId,
      parentAttributeValueId,
      propId,
      regionValue,
    );
  }

  // await sleepBetween(5000, 15000);

  // CREATE VPC
  const vpcVariant = await extractSchemaVariant(
    schemaVariants,
    "VPC",
    "AWS EC2",
  );
  const vpcVariantId = vpcVariant.schemaVariantId;

  const vpcComponentId = await createComponent(
    sdf,
    changeSetId,
    vpcVariantId,
    0,
    0,
    undefined,
    newCreateComponentApi,
  );

  // await sleepBetween(1000, 2000);

  await setComponentType(
    sdf,
    changeSetId,
    vpcComponentId,
    "configurationFrameDown",
  );

  await sleepBetween(0, 750);

  // CONFIGURE VPC
  const { values: vpcPropValues } = await getPropertyEditor(
    sdf,
    changeSetId,
    vpcComponentId,
  );

  for (const { p: path, v: value } of [
    { p: "/root/si/name", v: "How to VPC" },
    { p: "/root/domain/CidrBlock", v: "10.0.0.0/16" },
    { p: "/root/domain/EnableDnsHostnames", v: true },
    { p: "/root/domain/EnableDnsResolution", v: true },
  ]) {
    const { attributeValueId, parentAttributeValueId, propId } =
      attributeValueIdForPropPath(path, vpcVariant.props, vpcPropValues);

    await setAttributeValue(
      sdf,
      changeSetId,
      vpcComponentId,
      attributeValueId,
      parentAttributeValueId,
      propId,
      value,
    );
    await sleepBetween(0, 750);
  }

  // Public Subnet Components
  const subnetVariant = await extractSchemaVariant(
    schemaVariants,
    "Subnet",
    "AWS EC2",
  );
  const subnetVariantId = subnetVariant.schemaVariantId;

  for (const { index, data } of [
    { CidrBlock: "10.0.128.0/20", AvailabilityZone: "us-east-1a" },
    { CidrBlock: "10.0.144.0/20", AvailabilityZone: "us-east-1b" },
    { CidrBlock: "10.0.160.0/20", AvailabilityZone: "us-east-1c" },
  ].map((data, index) => ({ index, data }))) {
    const subnetComponentId = await createComponent(
      sdf,
      changeSetId,
      subnetVariantId,
      -550 + 550 * index,
      150,
      vpcComponentId,
      newCreateComponentApi,
    );
    await sleepBetween(1000, 5000);

    await setComponentType(
      sdf,
      changeSetId,
      subnetComponentId,
      "configurationFrameDown",
    );

    const { values: subnetPropValues } = await getPropertyEditor(
      sdf,
      changeSetId,
      subnetComponentId,
    );

    // CONFIGURE Subnet

    for (const { p: path, v: value } of [
      { p: "/root/si/name", v: `Public ${index + 1}` },
      { p: "/root/domain/CidrBlock", v: data.CidrBlock },
      { p: "/root/domain/AvailabilityZone", v: data.AvailabilityZone },
      { p: "/root/domain/IsPublic", v: true },
    ]) {
      const { attributeValueId, parentAttributeValueId, propId } =
        attributeValueIdForPropPath(
          path,
          subnetVariant.props,
          subnetPropValues,
        );

      await setAttributeValue(
        sdf,
        changeSetId,
        subnetComponentId,
        attributeValueId,
        parentAttributeValueId,
        propId,
        value,
      );

      // await sleepBetween(3000, 10000);
    }
  }

  await sdf.waitForDVURoots(changeSetId, 2000, 60000);

}

// REQUEST HELPERS WITH VALIDATIONS

async function getDiagram(
  sdf: SdfApiClient,
  changeSetId: string,
): Promise<{ components: any[]; edges: any[] }> {
  const diagram = await sdf.call({
    route: "get_diagram",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });

  assert(
    Array.isArray(diagram?.components),
    "Expected components list on the diagram",
  );
  assert(Array.isArray(diagram?.edges), "Expected edges list on the diagram");

  return diagram;
}

async function setComponentType(
  sdf: SdfApiClient,
  changeSetId: string,
  componentId: string,
  componentType:
    | "component"
    | "configurationFrameDown"
    | "configurationFrameUp",
) {
  const payload = {
    componentId,
    componentType,
    visibility_change_set_pk: changeSetId,
    requestUlid: changeSetId,
  };

  const result = await sdf.call({
    route: "set_component_type",
    body: payload,
  });

  // Make side effect calls
  await Promise.all([
    getActions(sdf, changeSetId),
    getFuncs(sdf, changeSetId),
    getPropertyEditor(sdf, changeSetId, componentId),
  ]);
  const res = getQualificationSummary(sdf, changeSetId);

  return result;
}


