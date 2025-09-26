async function main(component: Input): Promise<Output> {
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    throw new Error(
      "HETZNER_API_TOKEN not found (hint: you may need a secret)",
    );
  }

  const endpoint = _.get(component.properties, ["domain", "extra", "endpoint"], "");
  const id = component.properties?.domain?.id;

  if (!endpoint) {
    throw new Error("Endpoint not found in extra properties");
  }

  if (!id) {
    throw new Error("Resource ID not found in domain properties");
  }

  const response = await fetch(
    `https://api.hetzner.cloud/v1/${endpoint}/${id}`,
    {
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();
  const resource = data[endpoint.slice(0, -1)] || data; // Handle singular vs plural endpoint names

  const resourceId = resource.id?.toString() || resource.name || id;
  console.log(`Importing ${endpoint} ${resourceId}`);

  const create: Output["ops"]["create"] = {};
  const actions = {};

  const properties = {
    si: {
      resourceId,
      type: endpoint,
    },
    domain: {
      ...resource,
    },
    resource: resource,
  };

  const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
  for (const [skey, svalue] of Object.entries(component.sources || {})) {
    newAttributes[skey] = {
      $source: svalue,
    };
  }

  create[resourceId] = {
    kind: endpoint,
    properties,
    attributes: newAttributes,
  };
  actions[resourceId] = {
    remove: ["create"],
  };

  return {
    status: "ok",
    message: `Imported ${endpoint} ${resourceId}`,
    ops: {
      create,
      actions,
    },
  };
}
