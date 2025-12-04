async function main(component: Input): Promise<Output> {
  const tenantId = requestStorage.getEnv("ENTRA_TENANT_ID") ||
    requestStorage.getEnv("AZURE_TENANT_ID");
  const clientId = requestStorage.getEnv("ENTRA_CLIENT_ID") ||
    requestStorage.getEnv("AZURE_CLIENT_ID");
  const clientSecret = requestStorage.getEnv("ENTRA_CLIENT_SECRET") ||
    requestStorage.getEnv("AZURE_CLIENT_SECRET");

  if (!tenantId || !clientId || !clientSecret) {
    throw new Error("Microsoft credentials not found");
  }

  if (!component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Unable to queue an update action on a component without a resource",
    };
  }

  const resourceId = _.get(component.properties, ["si", "resourceId"]);
  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );
  const apiVersion = _.get(
    component.properties,
    ["domain", "extra", "apiVersion"],
    "v1.0",
  );

  if (!resourceId || !endpoint) {
    throw new Error("Missing resourceId or endpoint for update");
  }

  const token = await getGraphToken(tenantId, clientId, clientSecret);

  // First, get current resource state
  const resourceUrl =
    `https://graph.microsoft.com/${apiVersion}/${endpoint}/${resourceId}`;

  console.log(`[UPDATE] Fetching current state: GET ${resourceUrl}`);
  const refreshResponse = await fetch(resourceUrl, {
    method: "GET",
    headers: {
      "Authorization": `Bearer ${token}`,
    },
  });

  if (!refreshResponse.ok) {
    const errorText = await refreshResponse.text();
    console.error(`[UPDATE] Failed to fetch current state:`, errorText);
    return {
      status: "error",
      payload: component.properties.resource.payload,
      message: `Failed to fetch current state: ${refreshResponse.status} ${refreshResponse.statusText} - ${errorText}`,
    };
  }

  const currentState = await refreshResponse.json();
  console.log(`[UPDATE] Current state fetched`);

  // Clean the desired payload (excluding createOnly properties)
  const updatePayload = cleanPayload(component.properties.domain, true);

  console.log(`[UPDATE] Starting update operation for resourceId: ${resourceId}`);
  console.log(`[UPDATE] PATCH ${resourceUrl}`);

  // Graph API uses PATCH for updates
  const response = await fetch(resourceUrl, {
    method: "PATCH",
    headers: {
      "Authorization": `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(updatePayload),
  });

  console.log(`[UPDATE] Response status: ${response.status}`);

  if (!response.ok) {
    const errorText = await response.text();
    console.error(`[UPDATE] Failed with status ${response.status}:`, errorText);
    return {
      status: "error",
      payload: component.properties.resource.payload,
      message: `Graph API Error: ${response.status} ${response.statusText} - ${errorText}`,
    };
  }

  // Graph API PATCH typically returns 204 No Content or 200 with updated resource
  let result;
  if (response.status === 204) {
    // No content returned, fetch the updated resource
    console.log(`[UPDATE] 204 No Content, fetching updated resource`);
    const finalResponse = await fetch(resourceUrl, {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    if (!finalResponse.ok) {
      return {
        status: "error",
        payload: component.properties.resource.payload,
        message: `Failed to fetch updated resource: ${finalResponse.status} ${finalResponse.statusText}`,
      };
    }

    result = await finalResponse.json();
  } else {
    result = await response.json();
  }

  console.log(`[UPDATE] Update successful`);
  return {
    payload: result,
    status: "ok",
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

function cleanPayload(domain, isUpdate: boolean) {
  const propUsageMap = JSON.parse(domain.extra.PropUsageMap);
  if (
    !Array.isArray(propUsageMap.createOnly) ||
    !Array.isArray(propUsageMap.updatable)
  ) {
    throw Error("malformed propUsageMap on resource");
  }

  const payload = _.cloneDeep(domain);

  // Remove extra metadata
  delete payload.extra;

  // Merge discriminator subtypes into parent
  for (
    const [discriminatorProp, subtypeMap] of Object.entries(
      propUsageMap.discriminators || {},
    )
  ) {
    const discriminatorObject = payload[discriminatorProp];

    if (!discriminatorObject || typeof discriminatorObject !== "object") {
      continue;
    }

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
      delete payload[discriminatorProp];
      continue;
    }

    const subtypeName = filledSubtypes[0];
    const subtypeValue = discriminatorObject[subtypeName];

    if (
      subtypeValue && typeof subtypeValue === "object" &&
      !Array.isArray(subtypeValue)
    ) {
      Object.assign(payload, subtypeValue);
      payload[discriminatorProp] = subtypeMap[subtypeName];
    } else {
      delete payload[discriminatorProp];
    }
  }

  // Only check top-level properties
  const propsToVisit = _.keys(payload).map((k: string) => [k]);

  while (propsToVisit.length > 0) {
    const key = propsToVisit.pop();

    let parent = payload;
    let keyOnParent = key[0];
    for (let i = 1; i < key.length; i++) {
      parent = parent[key[i - 1]];
      keyOnParent = key[i];
    }

    if (key.length === 1) {
      const propPath = `/domain/${keyOnParent}`;
      const isCreateOnlyProp = propUsageMap.createOnly.includes(propPath);
      const isUpdatableProp = propUsageMap.updatable.includes(propPath);

      // For updates, exclude createOnly properties
      if (isUpdate && isCreateOnlyProp) {
        delete parent[keyOnParent];
        continue;
      }

      // For both create and update, exclude properties that aren't in either list
      if (!isCreateOnlyProp && !isUpdatableProp) {
        delete parent[keyOnParent];
        continue;
      }
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
