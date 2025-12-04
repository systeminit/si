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
    const payload = _.get(component, "properties.resource.payload");
    if (payload) {
      return {
        status: "error",
        payload,
        message: "Missing resourceId or endpoint for delete",
      };
    } else {
      return {
        status: "error",
        message: "Missing resourceId or endpoint for delete",
      };
    }
  }

  const token = await getGraphToken(tenantId, clientId, clientSecret);
  const url =
    `https://graph.microsoft.com/${apiVersion}/${endpoint}/${resourceId}`;

  console.log(`[DELETE] DELETE ${url}`);
  const response = await fetch(url, {
    method: "DELETE",
    headers: {
      "Authorization": `Bearer ${token}`,
    },
  });

  console.log(`[DELETE] Response status: ${response.status}`);

  if (!response.ok) {
    // 404 means resource is already deleted, which is fine
    if (response.status === 404) {
      console.log("Resource already deleted (404)");
      return {
        payload: null,
        status: "ok",
      };
    }

    const errorText = await response.text();
    console.error(`[DELETE] Failed with status ${response.status}:`, errorText);

    const payload = _.get(component, "properties.resource.payload");
    if (payload) {
      return {
        status: "error",
        payload,
        message: `Delete error: ${response.status} ${response.statusText} - ${errorText}`,
      };
    } else {
      return {
        status: "error",
        message: `Delete error: ${response.status} ${response.statusText} - ${errorText}`,
      };
    }
  }

  console.log(`[DELETE] Delete successful`);
  return {
    payload: null,
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
