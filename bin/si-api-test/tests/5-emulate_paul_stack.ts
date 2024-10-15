// deno-lint-ignore-file no-explicit-any
import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
  runWithTemporaryChangeset,
  sleep,
  sleepBetween,
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

  await setComponentGeometry(
    sdf,
    changeSetId,
    regionComponentId,
    0,
    0,
    1800,
    800,
  );
  //await sleepBetween(1000, 5000);

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

  await setComponentGeometry(
    sdf,
    changeSetId,
    vpcComponentId,
    0,
    80,
    1700,
    600,
    {
      newParent: regionComponentId,
    },
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

  await sdf.waitForDVUs(2000, 20000);
}

// REQUEST HELPERS WITH VALIDATIONS
async function createComponent(
  sdf: SdfApiClient,
  changeSetId: string,
  schemaVariantId: string,
  x: number,
  y: number,
  parentId?: string,
  newCreateComponentApi?: boolean,
): Promise<string> {
  const parentArgs = parentId ? { parentId } : {};
  const payload = {
    schemaType: newCreateComponentApi ? "installed" : undefined,
    schemaVariantId,
    x: x.toString(),
    y: y.toString(),
    visibility_change_set_pk: changeSetId,
    workspaceId: sdf.workspaceId,
    ...parentArgs,
  };
  const createResp = await sdf.call({
    route: "create_component",
    body: payload,
  });
  const componentId = createResp?.componentId;
  assert(componentId, "Expected to get a component id after creation");

  // Run side effect calls
  await Promise.all([
    getQualificationSummary(sdf, changeSetId),
    getActions(sdf, changeSetId),
    getFuncs(sdf, changeSetId),
    getPropertyEditor(sdf, changeSetId, componentId),
  ]);

  return componentId;
}

async function setComponentGeometry(
  sdf: SdfApiClient,
  changeSetId: string,
  componentId: string,
  x: number,
  y: number,
  w: number,
  h: number,
  parentArguments?: {
    newParent?: string;
    detach?: boolean;
  },
) {
  const someParentArguments = parentArguments ?? {};
  const setPositionPayload = {
    dataByComponentId: {
      [componentId]: {
        geometry: {
          x: x.toString(),
          y: y.toString(),
          width: w.toString(),
          height: h.toString(),
        },
        detach: false,
        ...someParentArguments,
      },
    },
    diagramKind: "configuration",
    visibility_change_set_pk: changeSetId,
    workspaceId: sdf.workspaceId,
    requestUlid: changeSetId,
    clientUlid: ulid(),
  };

  const result = await sdf.call({
    route: "set_component_position",
    body: setPositionPayload,
  });

  // Make side effect calls
  await Promise.all([
    getQualificationSummary(sdf, changeSetId),
    getActions(sdf, changeSetId),
    getFuncs(sdf, changeSetId),
  ]);

  return result;
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

async function setAttributeValue(
  sdf: SdfApiClient,
  changeSetId: string,
  componentId: string,
  attributeValueId: string,
  parentAttributeValueId: string,
  propId: string,
  value: unknown,
) {
  const updateValuePayload = {
    visibility_change_set_pk: changeSetId,
    componentId,
    attributeValueId,
    parentAttributeValueId,
    propId,
    value,
    isForSecret: false,
  };

  await sdf.call({
    route: "update_property_value",
    body: updateValuePayload,
  });
}

async function getSchemaVariants(sdf: SdfApiClient, changeSetId: string) {
  let schemaVariants = await sdf.call({
    route: "schema_variants",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });

  const newCreateComponentApi = Array.isArray(schemaVariants?.installed);
  if (newCreateComponentApi) {
    schemaVariants = schemaVariants.installed;
  }

  assert(
    Array.isArray(schemaVariants),
    "List schema variants should return an array",
  );

  return { schemaVariants, newCreateComponentApi };
}

async function getPropertyEditor(
  sdf: SdfApiClient,
  changeSetId: string,
  componentId: string,
) {
  const values = await sdf.call({
    route: "get_property_values",
    routeVars: {
      componentId,
      changeSetId,
    },
  });
  assert(typeof values?.values === "object", "Expected prop values");
  assert(typeof values?.childValues === "object", "Expected prop childValues:");

  const schema = await sdf.call({
    route: "get_property_schema",
    routeVars: {
      componentId,
      changeSetId,
    },
  });
  assert(typeof schema?.rootPropId === "string", "Expected rootPropId");
  assert(typeof schema?.props === "object", "Expected props");
  assert(typeof schema?.childProps === "object", "Expected childProps list");

  return {
    values,
    schema,
  };
}

async function getQualificationSummary(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "qualification_summary",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}

async function getActions(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "action_list",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}

async function getFuncs(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "func_list",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}

// Data Extractors
function extractSchemaVariant(
  schemaVariants: any[],
  schemaName: string,
  category?: string,
) {
  const variant = schemaVariants.find(
    (sv) =>
      sv.schemaName === schemaName && (!category || sv.category === category),
  );

  const awsRegionVariantId = variant?.schemaVariantId;
  assert(
    awsRegionVariantId,
    `Expected to find ${schemaName} schema and variant`,
  );

  return variant;
}

function attributeValueIdForPropPath(
  propPath: string,
  propList: any[],
  attributeValuesView: {
    values: any[];
    childValues: any[];
  },
) {
  const prop = propList.find((p) => p.path === propPath);
  assert(prop, `Expected to find ${propPath} prop`);

  let attributeValueId;
  for (const attributeValue in attributeValuesView.values) {
    if (attributeValuesView.values[attributeValue]?.propId === prop.id) {
      attributeValueId = attributeValue;
    }
  }
  assert(attributeValueId, "Expected source attribute value");

  let parentAttributeValueId;
  for (const attributeValue in attributeValuesView?.childValues) {
    const avChildren = attributeValuesView?.childValues[attributeValue] ?? [];
    if (avChildren.includes(attributeValueId)) {
      parentAttributeValueId = attributeValue;
    }
  }
  assert(parentAttributeValueId, "Expected parent of source attribute value");

  return {
    attributeValueId,
    parentAttributeValueId,
    propId: prop.id,
  };
}
