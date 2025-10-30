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
      message: "Could not delete, no resource present",
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
      message: "No resource ID found for delete",
    };
  }

  const token = await getAzureToken(tenantId, clientId, clientSecret);
  const url =
    `https://management.azure.com${resourceId}?api-version=${apiVersion}`;

  console.log(`DELETE ${url}`);
  const response = await fetch(url, {
    method: "DELETE",
    headers: {
      "Authorization": `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    if (response.status === 404) {
      console.log("Resource not found upstream (404), considering it deleted");
      return {
        status: "ok",
      };
    }

    const errorText = await response.text();
    throw new Error(
      `Azure API Error: ${response.status} ${response.statusText} - ${errorText}`,
    );
  }

  return {
    status: "ok",
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
