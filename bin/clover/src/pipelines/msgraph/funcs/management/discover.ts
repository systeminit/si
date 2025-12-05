async function main({
  thisComponent,
}: Input): Promise<Output> {
  const component = thisComponent;
  const tenantId = requestStorage.getEnv("ENTRA_TENANT_ID") ||
    requestStorage.getEnv("AZURE_TENANT_ID");
  const clientId = requestStorage.getEnv("ENTRA_CLIENT_ID") ||
    requestStorage.getEnv("AZURE_CLIENT_ID");
  const clientSecret = requestStorage.getEnv("ENTRA_CLIENT_SECRET") ||
    requestStorage.getEnv("AZURE_CLIENT_SECRET");

  if (!tenantId || !clientId || !clientSecret) {
    throw new Error("Microsoft credentials not found");
  }

  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );
  const resourceType = _.get(
    component.properties,
    ["domain", "extra", "EntraResourceType"],
    "",
  );
  const apiVersion = _.get(
    component.properties,
    ["domain", "extra", "apiVersion"],
    "v1.0",
  );
  const propUsageMapJson = _.get(
    component.properties,
    ["domain", "extra", "PropUsageMap"],
    "{}",
  );

  if (!endpoint) {
    return {
      status: "error",
      message: "Endpoint not found in domain.extra.endpoint",
    };
  }

  if (!resourceType) {
    return {
      status: "error",
      message: "EntraResourceType not found in domain.extra",
    };
  }

  // Parse PropUsageMap to get updatable properties
  let updatableProperties: Set<string>;
  let createOnlyProperties: Set<string>;
  let propUsageMap;
  try {
    propUsageMap = JSON.parse(propUsageMapJson);
    updatableProperties = new Set(propUsageMap.updatable || []);
    createOnlyProperties = new Set(propUsageMap.createOnly || []);
  } catch (e) {
    console.warn(
      `Failed to parse PropUsageMap for ${resourceType}, using empty set:`,
      e,
    );
    updatableProperties = new Set();
    createOnlyProperties = new Set();
    propUsageMap = {};
  }

  // Build refinement filter from non-empty domain properties
  const refinement: Record<string, any> = {};
  const domainProps = component.properties?.domain || {};
  for (const [key, value] of Object.entries(domainProps)) {
    // Skip extra metadata and empty values
    if (key === "extra" || value == null || value === "") {
      continue;
    }
    refinement[key] = value;
  }

  console.log(`Discovering Entra resources: ${endpoint}`);
  if (Object.keys(refinement).length > 0) {
    console.log("Refinement filter:", JSON.stringify(refinement, null, 2));
  }

  const token = await getGraphToken(tenantId, clientId, clientSecret);

  // List all resources with pagination
  let allResources = [];
  let nextLink = `https://graph.microsoft.com/${apiVersion}/${endpoint}`;

  while (nextLink) {
    const response = await fetch(nextLink, {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      return {
        status: "error",
        message:
          `Graph API Error: ${response.status} ${response.statusText} - ${errorText}`,
      };
    }

    const data = await response.json();
    const resources = data.value || [];
    allResources = allResources.concat(resources);

    nextLink = data["@odata.nextLink"] || null;

    if (nextLink) {
      console.log(`Fetching next page... (${allResources.length} resources so far)`);
    }
  }

  console.log(`Found ${allResources.length} total resources`);

  // Process each resource
  const createOps: Record<string, any> = {};
  const actionOps: Record<string, any> = {};
  let importedCount = 0;

  for (const resource of allResources) {
    // Extract resource ID (could be 'id' or other field)
    const resourceId = resource.id;
    if (!resourceId) {
      console.warn("Skipping resource without id:", resource);
      continue;
    }

    console.log(`Importing ${resourceId}`);

    // Fetch full resource details
    const detailUrl = `https://graph.microsoft.com/${apiVersion}/${endpoint}/${resourceId}`;
    const detailResponse = await fetch(detailUrl, {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    if (!detailResponse.ok) {
      console.warn(`Failed to fetch details for ${resourceId}: ${detailResponse.status}`);
      continue;
    }

    const fullResource = await detailResponse.json();

    // Transform Graph flat structure to SI nested structure
    const transformedResource = transformGraphToSI(fullResource, propUsageMap);

    // Build domain properties by only including updatable properties
    const resourceDomainProperties: Record<string, any> = {};

    for (const path of updatableProperties) {
      // Strip "/domain/" prefix to get the path within domain
      if (!path.startsWith("/domain/")) {
        continue;
      }
      const domainPath = path.substring("/domain/".length);

      // Get value from transformedResource using the path
      const value = _.get(transformedResource, domainPath);

      // Set in resourceDomainProperties if value exists
      if (value != null) {
        _.set(resourceDomainProperties, domainPath, value);
      }
    }

    // Apply refinement filter
    if (Object.keys(refinement).length > 0) {
      if (!_.isMatch(resourceDomainProperties, refinement)) {
        console.log(`Skipping ${resourceId} - doesn't match refinement filter`);
        continue;
      }
    }

    // Build attributes by filtering out createOnly subscriptions
    const newAttributes: Record<string, any> = {};
    for (const [skey, svalue] of Object.entries(component.sources || {})) {
      // Skip createOnly attributes
      if (createOnlyProperties.has(skey)) {
        continue;
      }
      newAttributes[skey] = {
        $source: svalue,
      };
    }

    // Create component entry
    const componentName = transformedResource.displayName ||
                          transformedResource.name ||
                          resourceId;

    createOps[resourceId] = {
      kind: resourceType,
      properties: {
        si: {
          name: componentName,
          resourceId: resourceId,
        },
        domain: {
          extra: {
            EntraResourceType: resourceType,
            endpoint: endpoint,
            apiVersion: apiVersion,
            PropUsageMap: propUsageMapJson,
          },
          ...resourceDomainProperties,
        },
        resource: transformedResource,
      },
      attributes: newAttributes,
    };

    // Remove create action for each discovered resource
    actionOps[resourceId] = {
      remove: ["create"],
    };

    importedCount++;
  }

  if (importedCount === 0) {
    return {
      status: "ok",
      message: `No resources found matching criteria for ${resourceType}`,
      ops: {},
    };
  }

  return {
    status: "ok",
    message: `Discovered ${importedCount} resource${importedCount !== 1 ? "s" : ""} of type ${resourceType}`,
    ops: {
      create: createOps,
      actions: actionOps,
    },
  };
}

async function getGraphToken(
  tenantId: string,
  clientId: string,
  clientSecret: string,
): Promise<string> {
  const tokenUrl =
    `https://login.microsoftonline.com/${tenantId}/oauth2/v2.0/token`;
  const body = new URLSearchParams({
    client_id: clientId,
    client_secret: clientSecret,
    scope: "https://graph.microsoft.com/.default",
    grant_type: "client_credentials",
  });

  const response = await fetch(tokenUrl, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: body.toString(),
  });

  if (!response.ok) {
    throw new Error(
      `Failed to get Graph API token: ${response.status} ${response.statusText}`,
    );
  }

  const data = await response.json();
  return data.access_token;
}

function transformGraphToSI(graphResource, propUsageMap) {
  const transformed = _.cloneDeep(graphResource);

  // Transform discriminators from flat to nested structure
  for (
    const [discriminatorProp, subtypeMap] of Object.entries(
      propUsageMap.discriminators || {},
    )
  ) {
    const discriminatorValue = transformed[discriminatorProp];

    if (!discriminatorValue || typeof discriminatorValue !== "string") {
      continue;
    }

    // Reverse lookup: find which subtype has this enum value
    const subtypeName = Object.entries(subtypeMap).find(
      ([_, enumValue]) => enumValue === discriminatorValue,
    )?.[0];

    if (!subtypeName) {
      continue;
    }

    // Get the properties that belong to this subtype
    const subtypeProps =
      propUsageMap.discriminatorSubtypeProps?.[discriminatorProp]
        ?.[subtypeName] || [];

    // Create nested structure
    const subtypeObject = {};
    for (const propName of subtypeProps) {
      if (propName in transformed) {
        subtypeObject[propName] = transformed[propName];
        delete transformed[propName];
      }
    }

    // Replace flat discriminator with nested structure
    transformed[discriminatorProp] = {
      [subtypeName]: subtypeObject,
    };
  }

  return transformed;
}
