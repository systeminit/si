async function main(component: Input): Promise<Output> {
  const token = requestStorage.getEnv("DO_API_TOKEN");
  if (!token) {
    return {
      status: "error",
      message: "DO_API_TOKEN not found (hint: you may need a secret)",
    };
  }

  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );

  const resourceId = component.properties?.si?.resourceId;

  if (!endpoint) {
    return {
      status: "error",
      message: "No endpoint found in domain configuration",
    };
  }

  if (!resourceId) {
    return {
      status: "error",
      message: "No resource ID found for deletion",
    };
  }

  // Construct URL - endpoint already starts with /v2/
  let url = `https://api.digitalocean.com${endpoint}/${resourceId}`;

  // Append any required query parameters from metadata
  const requiredQueryParamsJson = _.get(
    component.properties,
    ["domain", "extra", "RequiredQueryParams"],
    "[]",
  );
  const requiredQueryParams = JSON.parse(requiredQueryParamsJson);

  if (requiredQueryParams.length > 0) {
    const queryParts: string[] = [];
    for (const paramName of requiredQueryParams) {
      const paramValue = component.properties?.resource?.payload?.[paramName];
      if (paramValue) {
        queryParts.push(`${paramName}=${encodeURIComponent(paramValue)}`);
      }
    }
    if (queryParts.length > 0) {
      url += `?${queryParts.join("&")}`;
    }
  }

  const response = await fetch(
    url,
    {
      method: "DELETE",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    const errorText = await response.text();
    return {
      status: "error",
      message: `Unable to delete resource; API returned ${response.status} ${response.statusText}: ${errorText}`,
    };
  }

  return {
    status: "ok",
    payload: null,
  };
}
