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
      message: "Could not delete, no resourceId present",
    };
  }

  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );
  const id = component.properties?.resource?.payload.id;

  const response = await fetch(
    `https://api.hetzner.cloud/v1/${endpoint}/${id}`,
    {
      method: "DELETE",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }

  return {
    status: "ok",
  };
}
