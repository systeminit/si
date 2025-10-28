async function main(component: Input): Promise<Output> {
  const tenantId = requestStorage.getEnv("AZURE_TENANT_ID");
  const clientId = requestStorage.getEnv("AZURE_CLIENT_ID");
  const clientSecret = requestStorage.getEnv("AZURE_CLIENT_SECRET");

  if (!tenantId || !clientId || !clientSecret) {
    throw new Error("Azure credentials not found");
  }

  const resourceId = _.get(component.properties, ["si", "resourceId"]);
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

  let propUsageMap;
  try {
    propUsageMap = JSON.parse(propUsageMapJson);
  } catch (e) {
    console.warn(
      "Failed to parse PropUsageMap, discriminators may not work:",
      e,
    );
    propUsageMap = {};
  }

  if (!resourceId) {
    return {
      status: component.properties.resource?.status ?? "error",
      message: "Could not refresh, no resourceId present",
    };
  }

  const token = await getAzureToken(tenantId, clientId, clientSecret);
  const url =
    `https://management.azure.com${resourceId}?api-version=${apiVersion}`;

  const response = await fetch(url, {
    method: "GET",
    headers: {
      "Authorization": `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    if (response.status === 404) {
      console.log("Resource not found upstream (404), removing the resource");
      return {
        status: "ok",
        payload: null,
      };
    }

    const errorText = await response.text();
    throw new Error(
      `Azure API Error: ${response.status} ${response.statusText} - ${errorText}`,
    );
  }

  const result = await response.json();

  // Transform Azure flat structure to SI nested structure
  const transformedResult = transformAzureToSI(result, propUsageMap);

  return {
    status: "ok",
    payload: transformedResult,
    resourceId: transformedResult.id,
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

function transformAzureToSI(azureResource, propUsageMap) {
  const transformed = _.cloneDeep(azureResource);

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
