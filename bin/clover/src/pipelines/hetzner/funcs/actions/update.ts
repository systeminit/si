async function main(component: Input): Promise<Output> {
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    throw new Error(
      "HETZNER_API_TOKEN not found (hint: you may need a secret)",
    );
  }

  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "error",
      message: "Could not update, no resource present",
    };
  }

  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );
  const id = component.properties?.resource?.payload.id;
  const updatePayload = component.properties.domain;

  if (!endpoint) {
    return {
      status: "error",
      message: "No endpoint found in domain configuration",
    };
  }

  if (!id) {
    return {
      status: "error",
      message: "No resource ID found for update",
    };
  }

  const response = await fetch(
    `https://api.hetzner.cloud/v1/${endpoint}/${id}`,
    {
      method: "PUT",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(updatePayload),
    },
  );

  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }

  const result = await response.json();
  const noun = endpoint.slice(0, -1);
  return {
    payload: result[noun],
    status: "ok",
  };
}
