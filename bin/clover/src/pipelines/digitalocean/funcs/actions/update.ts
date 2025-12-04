async function main(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "error",
      message: "Could not update, no resource present",
    };
  }

  const codeString = component.properties.code?.["DigitalOcean Update Code Gen"]?.code;
  if (!codeString) {
    return {
      status: "error",
      message: "Could not find DigitalOcean Update Code Gen code for resource",
    };
  }

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

  const updateMethod = _.get(
    component.properties,
    ["domain", "extra", "UpdateMethod"],
    "PUT",
  );

  if (!endpoint) {
    return {
      status: "error",
      message: "No endpoint found in domain configuration",
    };
  }

  if (!resourceId) {
    return {
      status: "error",
      message: "No resource ID found for update",
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
      method: updateMethod,
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: codeString,
    },
  );

  if (!response.ok) {
    const errorText = await response.text();
    return {
      status: "error",
      message: `Unable to update resource; API returned ${response.status} ${response.statusText}: ${errorText}`,
    };
  }

  const responseJson = await response.json();
  const resourceKey = Object.keys(responseJson).find(key => key !== "links" && key !== "meta");
  const payload = resourceKey ? responseJson[resourceKey] : responseJson;

  return {
    payload,
    status: "ok",
  };
}
