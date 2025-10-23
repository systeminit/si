async function main(component: Input): Promise<Output> {
  const tenantId = requestStorage.getEnv("AZURE_TENANT_ID");
  const clientId = requestStorage.getEnv("AZURE_CLIENT_ID");
  const clientSecret = requestStorage.getEnv("AZURE_CLIENT_SECRET");

  if (!tenantId || !clientId || !clientSecret) {
    throw new Error("Azure credentials not found");
  }

  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "error",
      message: "Could not update, no resource present",
    };
  }

  const resourceId = _.get(component.properties, ["si", "resourceId"]);
  const apiVersion = _.get(
    component.properties,
    ["domain", "extra", "apiVersion"],
    "2023-01-01",
  );

  if (!resourceId) {
    return {
      status: "error",
      message: "No resource ID found for update",
    };
  }

  const token = await getAzureToken(tenantId, clientId, clientSecret);
  const url =
    `https://management.azure.com${resourceId}?api-version=${apiVersion}`;

  // First, GET the current resource state
  const getCurrentResponse = await fetch(url, {
    method: "GET",
    headers: {
      "Authorization": `Bearer ${token}`,
    },
  });

  if (!getCurrentResponse.ok) {
    const errorText = await getCurrentResponse.text();
    throw new Error(
      `Failed to get current resource state: ${getCurrentResponse.status} ${getCurrentResponse.statusText} - ${errorText}`,
    );
  }

  const currentResource = await getCurrentResponse.json();

  // Merge the desired changes with the current state
  const updatePayload = cleanPayload(component.properties.domain);
  const mergedPayload = _.merge({}, currentResource, updatePayload);

  // PUT the complete merged resource definition
  const response = await fetch(url, {
    method: "PUT",
    headers: {
      "Authorization": `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(mergedPayload),
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
    // For updates, only include updatable properties, not createOnly
    if (key.length === 1 && !propUsageMap.updatable.includes(keyOnParent)) {
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
