import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
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
  // CREATE COMPONENTS
  // get schema variant ids
  let schemaVariants = await sdf.call({
    route: "schema_variants",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
  let newCreateComponentApi = false;
  if (Array.isArray(schemaVariants?.installed)) {
    newCreateComponentApi = true;
    schemaVariants = schemaVariants.installed;
  }

  assert(
    Array.isArray(schemaVariants),
    "List schema variants should return an array",
  );

  const ec2InstanceVariant = schemaVariants.find((sv) =>
    sv.schemaName === "EC2 Instance"
  );
  const ec2InstanceVariantId = ec2InstanceVariant?.schemaVariantId;
  assert(
    ec2InstanceVariantId,
    "Expected to find EC2 Instance schema and variant",
  );

  const awsRegionVariant = schemaVariants.find((sv) =>
    sv.schemaName === "Region" && sv.category === "AWS"
  );

  const awsRegionVariantId = awsRegionVariant?.schemaVariantId;
  assert(
    awsRegionVariantId,
    "Expected to find Generic Frame schema and variant",
  );

  // create region component
  const createRegionPayload = {
    "schemaVariantId": awsRegionVariantId,
    "x": "-200",
    "y": "0",
    "visibility_change_set_pk": changeSetId,
    "workspaceId": sdf.workspaceId,
  };
  if (newCreateComponentApi) {
    createRegionPayload["schemaType"] = "installed";
  }

  const createRegionResp = await sdf.call({
    route: "create_component",
    body: createRegionPayload,
  });
  const regionComponentId = createRegionResp?.componentId;
  assert(regionComponentId, "Expected to get a component id after creation");

  // create instance component
  const createInstancePayload = {
    "schemaVariantId": ec2InstanceVariantId,
    "x": "200",
    "y": "0",
    "visibility_change_set_pk": changeSetId,
    "workspaceId": sdf.workspaceId,
  };
  if (newCreateComponentApi) {
    createInstancePayload["schemaType"] = "installed";
  }
  const createInstanceResp = await sdf.call({
    route: "create_component",
    body: createInstancePayload,
  });

  const instanceComponentId = createInstanceResp?.componentId;
  assert(instanceComponentId, "Expected to get a component id after creation");

  // Check that components exists on diagram
  const diagram = await sdf.call({
    route: "get_diagram",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });

  assert(diagram?.components, "Expected components list on the diagram");
  assert(
    diagram.components.length === 2,
    "Expected a single component on the diagram",
  );

  const regionComponentOnDiagram = diagram.components.find((c) =>
    c.id === regionComponentId
  );
  assert(regionComponentOnDiagram, "Expected to find region on the diagram");
  assert(
    regionComponentOnDiagram?.schemaVariantId === awsRegionVariantId,
    "Expected diagram component schema variant id to be correct",
  );

  const instanceComponentOnDiagram = diagram.components.find((c) =>
    c.id === instanceComponentId
  );
  assert(instanceComponentOnDiagram, "Expected to find region on the diagram");
  assert(
    instanceComponentOnDiagram?.schemaVariantId === ec2InstanceVariantId,
    "Expected diagram component schema variant id to be correct",
  );

  // CONNECT COMPONENTS
  const outputSocket = awsRegionVariant.outputSockets.find((s) =>
    s.name === "Region"
  );
  assert(outputSocket?.id, "Expected to find region output socket");
  const inputSocket = ec2InstanceVariant.inputSockets.find((s) =>
    s.name === "Region"
  );
  assert(inputSocket?.id, "Expected to find region input socket");

  const createConnectionPayload = {
    "fromComponentId": regionComponentId,
    "fromSocketId": outputSocket?.id,
    "toComponentId": instanceComponentId,
    "toSocketId": inputSocket?.id,
    "visibility_change_set_pk": changeSetId,
    "workspaceId": sdf.workspaceId,
  };

  await sdf.call({
    route: "create_connection",
    body: createConnectionPayload,
  });

  const diagramWithConnection = await sdf.call({
    route: "get_diagram",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });

  assert(diagramWithConnection?.edges, "Expected edges list on the diagram");
  assert(
    diagramWithConnection.edges.length === 1,
    "Expected a single edge on the diagram",
  );

  const edge = diagramWithConnection.edges[0];
  assert(
    edge.fromComponentId === regionComponentId &&
      edge.toComponentId === instanceComponentId,
    "Expected edge to be between the right components",
  );

  // UPDATE SOURCE VALUE
  // get source prop
  const sourceRegionProp = awsRegionVariant.props.find((p) =>
    p.path === "/root/domain/region"
  );
  assert(sourceRegionProp, "Expected to find source region prop");
  // get attribute values for region
  const sourcePropValues = await sdf.call({
    route: "get_property_values",
    routeVars: {
      componentId: regionComponentId,
      changeSetId,
    },
  });
  assert(typeof sourcePropValues?.values === "object", "Expected prop values");
  assert(
    typeof sourcePropValues?.childValues === "object",
    "Expected prop childValues:",
  );

  let sourceAttributeValue;
  for (const attributeValue in sourcePropValues?.values) {
    if (
      sourcePropValues?.values[attributeValue]?.propId === sourceRegionProp.id
    ) {
      sourceAttributeValue = attributeValue;
    }
  }
  assert(sourceAttributeValue, "Expected source attribute value");

  let sourceAttributeValueParent;
  for (const attributeValue in sourcePropValues?.childValues) {
    const avChildren = sourcePropValues?.childValues[attributeValue] ?? [];
    if (avChildren.includes(sourceAttributeValue)) {
      sourceAttributeValueParent = attributeValue;
    }
  }
  assert(
    sourceAttributeValueParent,
    "Expected parent of source attribute value",
  );

  const regionValue = "us-west-1";
  const updateValuePayload = {
    "attributeValueId": sourceAttributeValue,
    "parentAttributeValueId": sourceAttributeValueParent,
    "propId": sourceRegionProp.id,
    "componentId": regionComponentId,
    "value": regionValue,
    "isForSecret": false,
    "visibility_change_set_pk": changeSetId,
  };

  await sdf.call({
    route: "update_property_value",
    body: updateValuePayload,
  });

  // CONFIRM VALUE ON DESTINATION
  // get source prop
  const destinationRegionProp = ec2InstanceVariant.props.find((p) =>
    p.path === "/root/domain/region"
  );
  assert(destinationRegionProp, "Expected to find destination region prop");
  // Try getting the values with backoff, to account for DVU
  await retryWithBackoff(async () => {
    // get attribute values for region
    const destinationPropValues = await sdf.call({
      route: "get_property_values",
      routeVars: {
        componentId: instanceComponentId,
        changeSetId,
      },
    });
    assert(
      typeof destinationPropValues?.values === "object",
      "Expected prop values",
    );

    let destinationRegionValue;
    for (const valueId in destinationPropValues?.values) {
      const value = destinationPropValues?.values[valueId];
      if (value?.propId === destinationRegionProp.id) {
        destinationRegionValue = value;
      }
    }
    assert(destinationRegionValue, "Expected to find destination region value");
    assert(
      destinationRegionValue.value === regionValue,
      "Expected propagated value to match source",
    );
  });

  // delete connection between components 
    const deleteConnection = {
      "fromComponentId": regionComponentId,
      "fromSocketId": edge.fromSocketId,
      "toComponentId": instanceComponentId,
      "toSocketId": edge.toSocketId,
      "visibility_change_set_pk": changeSetId,
    };

    const deleteResponse = await sdf.call({
      route: "delete_connection",
      body: deleteConnection,
    });

  // check value is no longer the source value (should be unset)
  const diagramWithoutConnection = await sdf.call({
    route: "get_diagram",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });

  assert(diagramWithoutConnection?.edges, "Expected edges list on the diagram");
  assert(
    diagramWithoutConnection.edges.length === 0,
    "Expected no edges on the diagram",
  );

  const deleteComponentPayload = {
    componentIds: [instanceComponentId, regionComponentId],
    forceErase: false,
    visibility_change_set_pk: changeSetId,
    workspaceId: sdf.workspaceId,
  };
  await sdf.call({
    route: "delete_components",
    body: deleteComponentPayload,
  });

}
