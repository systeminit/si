async function main({
  thisComponent,
}: Input): Promise<Output> {
  const component = thisComponent;
  const tenantId = requestStorage.getEnv("AZURE_TENANT_ID");
  const clientId = requestStorage.getEnv("AZURE_CLIENT_ID");
  const clientSecret = requestStorage.getEnv("AZURE_CLIENT_SECRET");

  if (!tenantId || !clientId || !clientSecret) {
    throw new Error("Azure credentials not found");
  }

  const subscriptionId = _.get(
    component.properties,
    ["domain", "subscriptionId"],
    "",
  );
  const resourceGroup = _.get(
    component.properties,
    ["domain", "resourceGroup"],
    "",
  );
  const resourceType = _.get(
    component.properties,
    ["domain", "extra", "AzureResourceType"],
    "",
  );
  const apiVersion = _.get(
    component.properties,
    ["domain", "extra", "apiVersion"],
    "2023-01-01",
  );
  const propUsageMapJson = _.get(
    component.properties,
    ["domain", "extra", "PropUsageMap"],
    "{}",
  );

  if (!subscriptionId) {
    return {
      status: "error",
      message: "subscriptionId is required in domain",
    };
  }

  if (!resourceType) {
    return {
      status: "error",
      message: "AzureResourceType not found in domain.extra",
    };
  }

  // Parse PropUsageMap to get updatable properties
  let updatableProperties: Set<string>;
  let createOnlyProperties: Set<string>;
  try {
    const propUsageMap = JSON.parse(propUsageMapJson);
    updatableProperties = new Set(propUsageMap.updatable || []);
    createOnlyProperties = new Set(propUsageMap.createOnly || []);
  } catch (e) {
    console.warn(
      `Failed to parse PropUsageMap for ${resourceType}, using empty set:`,
      e,
    );
    updatableProperties = new Set();
    createOnlyProperties = new Set();
  }

  console.log(`Discovering ${resourceType} resources...`);

  const token = await getAzureToken(tenantId, clientId, clientSecret);

  // Build refinement filter from domain properties
  const refinement = _.cloneDeep(thisComponent.properties.domain);
  delete refinement["extra"];
  delete refinement["location"];
  // Remove any empty values, as they are never refinements
  for (const [key, value] of Object.entries(refinement)) {
    if (_.isEmpty(value)) {
      delete refinement[key];
    } else if (_.isPlainObject(value)) {
      refinement[key] = _.pickBy(
        value,
        (v) => !_.isEmpty(v) || _.isNumber(v) || _.isBoolean(v),
      );
      if (_.isEmpty(refinement[key])) {
        delete refinement[key];
      }
    }
  }

  const listUrl =
    `https://management.azure.com/subscriptions/${subscriptionId}/providers/${resourceType}?api-version=${apiVersion}`;

  // Handle pagination with nextLink
  let resources: any[] = [];
  let nextLink: string | null = listUrl;

  while (nextLink) {
    const listResponse = await fetch(nextLink, {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    if (!listResponse.ok) {
      const errorText = await listResponse.text();
      return {
        status: "error",
        message:
          `Azure API Error: ${listResponse.status} ${listResponse.statusText} - ${errorText}`,
      };
    }

    const listData = await listResponse.json();
    resources = resources.concat(listData.value || []);
    nextLink = listData.nextLink || null;

    if (nextLink) {
      console.log(`Fetching next page: ${nextLink}`);
    }
  }

  console.log(`Found ${resources.length} resources`);

  const create: Output["ops"]["create"] = {};
  const actions = {};
  let importCount = 0;

  for (const resource of resources) {
    const resourceId = resource.id;

    console.log(`Importing ${resourceId}`);

    // Fetch the full resource details
    const resourceUrl =
      `https://management.azure.com${resourceId}?api-version=${apiVersion}`;
    const resourceResponse = await fetch(resourceUrl, {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    if (!resourceResponse.ok) {
      console.log(
        `Failed to fetch ${resourceId}, skipping (status: ${resourceResponse.status})`,
      );
      continue;
    }

    const fullResource = await resourceResponse.json();

    // Build domain by only including updatable properties from the resource
    // CreateOnly properties are immutable on existing resources, so we don't copy them
    const domainProperties: Record<string, any> = {
      subscriptionId,
      resourceGroup,
    };

    // Copy updatable properties from the resource
    for (const [key, value] of Object.entries(fullResource)) {
      if (updatableProperties.has(key) && value != null) {
        domainProperties[key] = value;
      }
    }

    const properties = {
      si: {
        resourceId,
      },
      domain: {
        ...domainProperties,
        extra: component.properties?.domain?.extra || {
          AzureResourceType: resourceType,
          apiVersion: apiVersion,
        },
      },
      resource: fullResource,
    };

    // Apply refinement filter
    if (_.isEmpty(refinement) || _.isMatch(properties.domain, refinement)) {
      const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
      for (const [skey, svalue] of Object.entries(component.sources || {})) {
        // Skip createOnly attributes - they can only be set on new components
        // Extract the property name from the path (e.g., "/domain/location" -> "location")
        const propName = skey.split("/").pop();
        if (propName && createOnlyProperties.has(propName)) {
          continue;
        }
        newAttributes[skey] = {
          $source: svalue,
        };
      }

      create[resourceId] = {
        kind: resourceType,
        properties,
        attributes: newAttributes,
      };
      actions[resourceId] = {
        remove: ["create"],
      };
      importCount++;
    } else {
      console.log(
        `Skipping import of ${resourceId}; it did not match refinements`,
      );
    }
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} ${resourceType} resources`,
    ops: {
      create,
      actions,
    },
  };
}

async function getAzureToken(
  tenantId: string,
  clientId: string,
  clientSecret: string,
): Promise<string> {
  const tokenUrl =
    `https://login.microsoftonline.com/${tenantId}/oauth2/v2.0/token`;
  const body = new URLSearchParams({
    client_id: clientId,
    client_secret: clientSecret,
    scope: "https://management.azure.com/.default",
    grant_type: "client_credentials",
  });

  const response = await fetch(tokenUrl, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: body.toString(),
  });

  if (!response.ok) {
    throw new Error(
      `Failed to get Azure token: ${response.status} ${response.statusText}`,
    );
  }

  const data = await response.json();
  return data.access_token;
}
