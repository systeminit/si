async function main(component: Input): Promise<Output> {
  const tenantId = requestStorage.getEnv("AZURE_TENANT_ID");
  const clientId = requestStorage.getEnv("AZURE_CLIENT_ID");
  const clientSecret = requestStorage.getEnv("AZURE_CLIENT_SECRET");

  if (!tenantId || !clientId || !clientSecret) {
    throw new Error("Azure credentials not found");
  }

  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  const domain = component.properties.domain;
  const subscriptionId = domain?.subscriptionId;
  const resourceGroup = domain?.resourceGroup;
  const resourceName = domain?.name;
  const resourceType = _.get(domain, ["extra", "AzureResourceType"], "");
  const apiVersion = _.get(domain, ["extra", "apiVersion"], "2023-01-01");

  if (!subscriptionId) {
    return {
      status: "error",
      message: "subscriptionId is required",
    };
  }

  if (!resourceGroup) {
    return {
      status: "error",
      message: "resourceGroup is required",
    };
  }

  if (!resourceName) {
    return {
      status: "error",
      message: "name is required",
    };
  }

  if (!resourceType) {
    return {
      status: "error",
      message: "AzureResourceType not found in domain.extra",
    };
  }

  const resourceId =
    `/subscriptions/${subscriptionId}/resourceGroups/${resourceGroup}/providers/${resourceType}/${resourceName}`;
  const token = await getAzureToken(tenantId, clientId, clientSecret);
  const url =
    `https://management.azure.com${resourceId}?api-version=${apiVersion}`;

  const createPayload = cleanPayload(domain);

  const response = await fetch(url, {
    method: "PUT",
    headers: {
      "Authorization": `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(createPayload),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      `Azure API Error: ${response.status} ${response.statusText} - ${errorText}`,
    );
  }

  const result = await response.json();

  return {
    payload: result,
    resourceId: result.id,
    status: "ok",
  };
}

function cleanPayload(domain) {
  const propUsageMap = JSON.parse(domain.extra.PropUsageMap);
  if (
    !Array.isArray(propUsageMap.createOnly) ||
    !Array.isArray(propUsageMap.updatable)
  ) {
    throw Error("malformed propUsageMap on resource");
  }

  const payload = _.cloneDeep(domain);

  // Remove metadata fields that are used for URL construction only
  delete payload.subscriptionId;
  delete payload.resourceGroup;
  delete payload.name;
  delete payload.extra;

  // Merge discriminator subtypes into parent
  // propUsageMap.discriminators maps discriminator property -> subtype name -> enum value
  // e.g., { "kind": { "AzurePowerShellScript": "AzurePowerShell", "AzureCliScript": "AzureCLI" } }
  for (
    const [discriminatorProp, subtypeMap] of Object.entries(
      propUsageMap.discriminators || {},
    )
  ) {
    const discriminatorObject = payload[discriminatorProp];

    if (!discriminatorObject || typeof discriminatorObject !== "object") {
      continue;
    }

    // Find which subtype is filled in
    const filledSubtypes = Object.keys(subtypeMap).filter((subtype) =>
      discriminatorObject[subtype]
    );

    if (filledSubtypes.length > 1) {
      throw new Error(
        `Multiple discriminator subtypes filled for "${discriminatorProp}": ${
          filledSubtypes.join(", ")
        }. Only one should be filled.`,
      );
    }

    if (filledSubtypes.length === 0) {
      // No subtype filled in, remove the discriminator property
      delete payload[discriminatorProp];
      continue;
    }

    const subtypeName = filledSubtypes[0];
    const subtypeValue = discriminatorObject[subtypeName];

    if (
      subtypeValue && typeof subtypeValue === "object" &&
      !Array.isArray(subtypeValue)
    ) {
      // Merge subtype properties into parent (subtype values take precedence)
      Object.assign(payload, subtypeValue);

      // Set the discriminator property to the enum value from the map
      payload[discriminatorProp] = subtypeMap[subtypeName];
    } else {
      // If no subtype data, remove the discriminator property
      delete payload[discriminatorProp];
    }
  }

  // Only check top-level properties - once a property is included, keep all its descendants
  const propsToVisit = _.keys(payload).map((k: string) => [k]);

  while (propsToVisit.length > 0) {
    const key = propsToVisit.pop();

    let parent = payload;
    let keyOnParent = key[0];
    for (let i = 1; i < key.length; i++) {
      parent = parent[key[i - 1]];
      keyOnParent = key[i];
    }

    // Only check against propUsageMap for top-level domain properties
    if (
      key.length === 1 &&
      !propUsageMap.createOnly.includes(keyOnParent) &&
      !propUsageMap.updatable.includes(keyOnParent)
    ) {
      delete parent[keyOnParent];
      continue;
    }

    const prop = parent[keyOnParent];

    if (typeof prop !== "object" || Array.isArray(prop)) {
      continue;
    }

    for (const childKey of _.keys(prop)) {
      propsToVisit.unshift([...key, childKey]);
    }
  }

  return extLib.removeEmpty(payload);
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
