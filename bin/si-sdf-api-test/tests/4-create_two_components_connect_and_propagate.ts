import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
  createComponent,
  createComponentPayload,
  eventualMVAssert,
  getVariants,
  getViews,
  retryWithBackoff,
  runWithTemporaryChangeset,
} from "../test_helpers.ts";

export default function create_two_components_connect_and_propagate(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  if (changeSetId) {
    return create_two_components_connect_and_propagate_inner(
      sdfApiClient,
      changeSetId,
    );
  } else {
    return runWithTemporaryChangeset(
      sdfApiClient,
      create_two_components_connect_and_propagate_inner,
    );
  }
}

async function create_two_components_connect_and_propagate_inner(
  sdf: SdfApiClient,
  changeSetId: string,
) {
  // Get the schema variants (uninstalled and installed)
  let schemaVariants = await getVariants(sdf, changeSetId);

  // Get the views and find the default one
  const views = await getViews(sdf, changeSetId);
  const defaultView = views.find((v: any) => v.isDefault);
  assert(defaultView, "Expected to find a default view");

  // Create two components, an EC2 Instance and a Region
  let createEC2ComponentBody = createComponentPayload(schemaVariants, "AWS::EC2::Instance");
  let createRegionComponentBody = createComponentPayload(schemaVariants, "Region");
  const newEC2ComponentId = await createComponent(sdf, changeSetId, defaultView.id, createEC2ComponentBody);
  assert(newEC2ComponentId, "Expected to get a component id after creation");
  const newRegionComponentId = await createComponent(sdf, changeSetId, defaultView.id, createRegionComponentBody);
  assert(newRegionComponentId, "Expected to get a component id after creation");

  // Subscribe EC2 Instance to Region
  const subResponse = await sdf.call({
    route: "attributes",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      componentId: newEC2ComponentId,
    },
    body: {
      "/domain/extra/Region": {
        "$source": {
          "component": newRegionComponentId,
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
    newEC2ComponentId,
    (mv) => Object.values(mv.attributeValues).some(
      (av: any) => av.path === "/domain/extra/Region" &&
        av.externalSources.length === 1,
    ),
    "Expected EC2 Instance to be subscribed to Region",
  );

  // Update Region value and check propagation
  const regionValue = "us-west-1";
  const updateRegionResponse = await sdf.call({
    route: "attributes",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      componentId: newRegionComponentId,
    },
    body: {
      "/domain/region": regionValue,
    },
  }, true);
  assert(updateRegionResponse.status == 200, "Expected to update region value successfully");

  // Check that the value was updated on the Region component
  await eventualMVAssert(
    sdf,
    changeSetId,
    "AttributeTree",
    newRegionComponentId,
    (mv) => Object.values(mv.attributeValues).some(
      (av: any) => av.path === "/domain/region" &&
        av.value === regionValue,
    ),
    "Expected Region to have a new value",
  );

  // check that the value was propagated to the EC2 Instance
  await eventualMVAssert(
    sdf,
    changeSetId,
    "AttributeTree",
    newEC2ComponentId,
    (mv) =>{
      const av = Object.values(mv.attributeValues).find((av:any)=> av.path === "/domain/extra/Region");
      console.log("AV IS: ", av);
      return Object.values(mv.attributeValues).some(
        (av: any) => av.path === "/domain/extra/Region" &&
          av.value === regionValue,
      )}
    ,
    "Expected propagated region value on EC2 Instance to match source",
    90000, // give it 90 seconds to make it's way through dvu
  );

  // Now remove the subscription and verify the value is no longer propagated
  const removeSubResponse = await sdf.call({
    route: "attributes",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      componentId: newEC2ComponentId,
    },
    body: {
      "/domain/extra/Region": { "$source": null },
    },
  });
  await eventualMVAssert(
    sdf,
    changeSetId,
    "AttributeTree",
    newEC2ComponentId,
    (mv) =>
      Object.values(mv.attributeValues).some(
        (av: any) => av.path === "/domain/extra/Region" &&
          !av.value && (!av.externalSources || av.externalSources.length === 0),
      )
    ,
    "Expected region value to no longer be propagated to EC2 Instance",
  );

  // lastly, delete both components
  const deleteComponentPayload = {
    componentIds: [newEC2ComponentId, newRegionComponentId],
    forceErase: false,
  };
  await sdf.call({
    route: "delete_components_v2",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
    body: deleteComponentPayload,
  });
  await eventualMVAssert(
    sdf,
    changeSetId,
    "ComponentList",
    sdf.workspaceId,
    (mv) => mv.components.length === 0,
    "Should be no components after deletion"
  );

}
