async function main({
  thisComponent,
}: Input): Promise<Output> {
  const component = thisComponent;
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    throw new Error(
      "HETZNER_API_TOKEN not found (hint: you may need a secret)",
    );
  }

  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );
  const resourceType = _.get(
    component.properties,
    ["domain", "extra", "HetznerResourceType"],
    "",
  );
  const resourceId = _.get(component.properties, ["si", "resourceId"], "");

  if (!endpoint) {
    throw new Error("Endpoint not found in extra properties");
  }

  if (!resourceType) {
    throw new Error("HetznerResourceType not found in extra properties");
  }

  if (!resourceId) {
    throw new Error("Resource ID not found in si properties");
  }

  const response = await fetch(
    `https://api.hetzner.cloud/v1/${endpoint}/${resourceId}`,
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
  const resource = data[endpoint.slice(0, -1)] || data;

  console.log(`Importing ${endpoint} ${resourceId}`);

  const create: Output["ops"]["create"] = {};
  const actions = {};

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
  for (const [skey, svalue] of Object.entries(component.sources || {})) {
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

  return {
    status: "ok",
    message: `Imported ${endpoint} ${resourceId}`,
    ops: {
      create,
      actions,
    },
  };
}
