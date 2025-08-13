// deno-lint-ignore-file no-explicit-any
import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
  runWithTemporaryChangeset,
  sleepBetween,
  createComponent,
  getVariants,
  createComponentPayload,
  getViews,
  eventualMVAssert,
} from "../test_helpers.ts";

export default async function emulate_paul_stack(sdfApiClient: SdfApiClient) {
  await sleepBetween(0, 750);
  return runWithTemporaryChangeset(sdfApiClient, emulate_paul_stack_inner);
}

async function emulate_paul_stack_inner(
  sdf: SdfApiClient,
  changeSetId: string,
) {
  // Get the schema variants (uninstalled and installed)
  let schemaVariants = await getVariants(sdf, changeSetId);
  let createRegionBody = createComponentPayload(schemaVariants, "Region");

  // Get the views and find the default one
  const views = await getViews(sdf, changeSetId);
  const defaultView = views.find((v: any) => v.isDefault);
  assert(defaultView, "Expected to find a default view");

  // Create a region component
  const regionComponentId = await createComponent(
    sdf,
    changeSetId,
    defaultView.id,
    createRegionBody,
  );
  // update region prop
  await sdf.call({
    route: "attributes",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      componentId: regionComponentId,
    },
    body: {
      "/domain/region": "us-east-1",
    },
  });



  // CREATE VPC
  let createVPCBody = createComponentPayload(schemaVariants, "AWS::EC2::VPC");
  const vpcComponentId = await createComponent(
    sdf,
    changeSetId,
    defaultView.id,
    createVPCBody,
  );

  // subscribe vpc to region component
  const subResponse = await sdf.call({
    route: "attributes",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      componentId: vpcComponentId,
    },
    body: {
      "/domain/extra/Region": {
        "$source": {
          "component": regionComponentId,
          "path": "/domain/region",
        }
      },
    },
  });

  // Check that the subscription was successful
  await eventualMVAssert(
    sdf,
    changeSetId,
    "AttributeTree",
    vpcComponentId,
    (mv) => Object.values(mv.attributeValues).some(
      (av: any) => av.path === "/domain/extra/Region" &&
        av.externalSources.length === 1,
    ),
    "Expected VPC to be subscribed to Region",
  );


  // CONFIGURE VPC
  const updateVpcResponse = await sdf.call({
    route: "attributes",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      componentId: vpcComponentId,
    },
    body: {
      "/si/name": "How to VPC",
      "/domain/CidrBlock": "10.0.0.0/16",
      "/domain/EnableDnsHostnames": true,
      "/domain/EnableDnsSupport": true,
    },
  });
  // Check that the value was updated on the VPC component
  await eventualMVAssert(
    sdf,
    changeSetId,
    "AttributeTree",
    vpcComponentId,
    (mv) => Object.values(mv.attributeValues).some(
      (av: any) => av.path === "/domain/CidrBlock" &&
        av.value === "10.0.0.0/16"
    ),
    "Expected VPC to have CidrBlock set to 10.0.0.0/16"
  );


  // Public Subnet Components
  const createSubnetBody = createComponentPayload(schemaVariants, "AWS::EC2::Subnet");


  for (const { index, data } of [
    { CidrBlock: "10.0.128.0/20", AvailabilityZone: "us-east-1a" },
    { CidrBlock: "10.0.144.0/20", AvailabilityZone: "us-east-1b" },
    { CidrBlock: "10.0.160.0/20", AvailabilityZone: "us-east-1c" },
  ].map((data, index) => ({ index, data }))) {
    const subnetComponentId = await createComponent(
      sdf,
      changeSetId,
      defaultView.id,
      createSubnetBody,
    );
    await sleepBetween(1000, 5000);
    // subscribe subnet to region and vpc components
    await sdf.call({
      route: "attributes",
      routeVars: {
        workspaceId: sdf.workspaceId,
        changeSetId,
        componentId: subnetComponentId,
      },
      body: {
        "/domain/extra/Region": {
          "$source": {
            "component": regionComponentId,
            "path": "/domain/region",
          },
        },
        "/domain/VpcId": {
          "$source": {
            "component": vpcComponentId,
            "path": "/resource_value/VpcId",
          },
        },
      },
    });
    // Check that the subscription was successful
    await eventualMVAssert(
      sdf,
      changeSetId,
      "AttributeTree",
      subnetComponentId,
      (mv) => Object.values(mv.attributeValues).some(
        (av: any) => av.path === "/domain/extra/Region" &&
          av.externalSources.length === 1 &&
          av.externalSources[0].path === "/domain/region",
      ),
      "Expected Subnet to be subscribed to Region",
    );

    // CONFIGURE Subnet

    for (const { p: path, v: value } of [
      { p: "/si/name", v: `Public ${index + 1}` },
      { p: "/domain/CidrBlock", v: data.CidrBlock },
      { p: "/domain/AvailabilityZone", v: data.AvailabilityZone },
      { p: "/domain/MapPublicIpOnLaunch", v: true },
    ]) {
      await sdf.call({
        route: "attributes",
        routeVars: {
          workspaceId: sdf.workspaceId,
          changeSetId,
          componentId: subnetComponentId,
        },
        body: {
          [path]: value,
        },
      });
    }
  }

  // update the region property on the region component to trigger propagation
  await sdf.call({
    route: "attributes",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      componentId: regionComponentId,
    },
    body: {
      "/domain/region": "us-east-1",
    },
  });
  await sdf.waitForDVURoots(changeSetId, 500, 30000);
  // verify that the region value propagated to the vpc and subnets
  const componentsToCheck = await sdf.mjolnir(changeSetId, "ComponentList", sdf.workspaceId);
  const componentIds = componentsToCheck.components.map((c) => c.id);
  for (const id of componentIds) {
    await eventualMVAssert(
      sdf,
      changeSetId,
      "AttributeTree",
      id,
      (mv) => Object.values(mv.attributeValues).some(
        (av: any) => av.path === "/domain/extra/Region" &&
          av.value === "us-east-1",
      ) || Object.values(mv.attributeValues).some(
        (av: any) => av.path === "/domain/region" &&
          av.value === "us-east-1",
      ),
      `Expected component ${id} to have region set to us-east-1`,
    );
  }
}



