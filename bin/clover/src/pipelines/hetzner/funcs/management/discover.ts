async function main({ thisComponent }: Input): Promise<Output> {
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    throw new Error(
      "HETZNER_API_TOKEN not found (hint: you may need a secret)",
    );
  }

  const endpoint = _.get(thisComponent.properties, ["domain", "extra", "endpoint"], "");
  const resourceType = _.get(thisComponent.properties, ["domain", "extra", "HetznerResourceType"], "");

  if (!endpoint) {
    throw new Error("Endpoint not found in extra properties");
  }

  if (!resourceType) {
    throw new Error("HetznerResourceType not found in extra properties");
  }

  const create: Output["ops"]["create"] = {};
  const actions = {};

  const response = await fetch(
    `https://api.hetzner.cloud/v1/${endpoint}`,
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
  const resources = data[endpoint] || [];
  let importCount = 0;

  for (const resource of resources) {
    const resourceId = resource.id?.toString() || resource.name;
    console.log(`Importing ${endpoint} ${resourceId}`);

    const properties = {
      si: {
        resourceId,
        type: resourceType,
      },
      domain: {
        ...resource,
      },
      resource: resource,
    };

    const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
    for (const [skey, svalue] of Object.entries(thisComponent.sources)) {
      newAttributes[skey] = {
        $source: svalue,
      };
    }

    create[resourceId] = {
      kind: resourceType,
      properties,
      attributes: newAttributes,
    };
    actions[resourceId] = {
      remove: ["create"],
    };
    importCount += 1;
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} ${endpoint}`,
    ops: {
      create,
      actions,
    },
  };
}
