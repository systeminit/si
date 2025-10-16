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

  const resourceId = _.get(component.properties, ["si", "resourceId"], "");
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

  if (!resourceId) {
    return {
      status: "error",
      message: "Resource ID not found in si.resourceId",
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

  console.log(`Importing Azure resource: ${resourceId}`);

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
    const errorText = await response.text();
    return {
      status: "error",
      message:
        `Azure API Error: ${response.status} ${response.statusText} - ${errorText}`,
    };
  }

  const resource = await response.json();

  // Build domain properties by only including writable properties from the resource
  const resourceDomainProperties: Record<string, any> = {
    subscriptionId: _.get(
      component.properties,
      ["domain", "subscriptionId"],
      "",
    ),
    resourceGroup: _.get(component.properties, ["domain", "resourceGroup"], ""),
  };

  // Copy updatable properties from the resource
  for (const [key, value] of Object.entries(resource)) {
    if (updatableProperties.has(key) && value != null) {
      resourceDomainProperties[key] = value;
    }
  }

  const resourcePayload = _.get(
    component.properties,
    ["resource", "payload"],
    "",
  );

  // Merge properties: start with existing, overlay resource properties
  const properties = {
    ...component.properties,
    domain: {
      extra: component.properties?.domain?.extra || {
        AzureResourceType: resourceType,
        apiVersion: apiVersion,
      },
      ...component.properties?.domain,
      ...resourceDomainProperties,
    },
  };

  // Only set resource if there's no existing payload
  let needsRefresh = true;
  if (!resourcePayload) {
    properties.resource = resource;
    needsRefresh = false;
  }

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

  const ops = {
    update: {
      self: {
        properties,
        attributes: newAttributes,
      },
    },
    actions: {
      self: {
        remove: ["create"],
        add: [] as string[],
      },
    },
  };

  if (needsRefresh) {
    ops.actions.self.add.push("refresh");
  } else {
    ops.actions.self.remove.push("refresh");
  }

  return {
    status: "ok",
    message: `Imported ${resourceType}: ${resource.name}`,
    ops,
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
