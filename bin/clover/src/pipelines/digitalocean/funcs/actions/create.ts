async function main(component: Input): Promise<Output> {
  const existingPayload = component.properties.resource?.payload;
  if (existingPayload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: existingPayload,
    };
  }

  const codeString = component.properties.code?.["DigitalOcean Create Code Gen"]?.code;
  if (!codeString) {
    return {
      status: "error",
      message: "Could not find DigitalOcean Create Code Gen code for resource",
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

  if (!endpoint) {
    return {
      status: "error",
      message: "No endpoint found in domain configuration",
    };
  }

  // Construct URL - endpoint already starts with /v2/
  const url = `https://api.digitalocean.com${endpoint}`;

  const response = await fetch(
    url,
    {
      method: "POST",
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
      message: `Unable to create resource; API returned ${response.status} ${response.statusText}: ${errorText}`,
    };
  }

  const responseJson = await response.json();

  // DigitalOcean wraps responses, extract the actual resource data
  // The response structure is { "droplet": {...} } or { "droplets": [...] }
  const resourceKey = Object.keys(responseJson).find(key => key !== "links" && key !== "meta");
  const payload = resourceKey ? responseJson[resourceKey] : responseJson;

  // Handle array responses (e.g., when creating multiple droplets)
  const actualPayload = Array.isArray(payload) ? payload[0] : payload;

  // Extract resource ID using the identifier field
  const identifierField = _.get(
    component.properties,
    ["domain", "extra", "IdentifierField"],
    "id",
  );

  const resourceId = actualPayload?.[identifierField];

  if (resourceId) {
    return {
      resourceId: resourceId.toString(),
      status: "ok",
      payload: actualPayload,
    };
  } else {
    return {
      message: `Failed to extract ${identifierField} from response`,
      status: "error",
      payload: actualPayload,
    };
  }
}
