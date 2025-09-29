async function main(component: Input): Promise<Output> {
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    throw new Error(
      "HETZNER_API_TOKEN not found (hint: you may need a secret)",
    );
  }

  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );
  const createPayload = cleanPayload(component.properties.domain);

  if (!endpoint) {
    return {
      status: "error",
      message: "No endpoint found in domain configuration",
    };
  }

  const response = await fetch(
    `https://api.hetzner.cloud/v1/${endpoint}`,
    {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(createPayload),
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

function cleanPayload(domain) {
  const propUsageMap = JSON.parse(domain.extra.PropUsageMap);
  if (
    !Array.isArray(propUsageMap.createOnly) ||
    !Array.isArray(propUsageMap.updatable)
  ) {
    throw Error("malformed propUsageMap on resource");
  }

  const payload = _.cloneDeep(domain);

  const propsToVisit = _.keys(payload).map((k: string) => [k]);

  // Visit the prop tree, deleting values that shouldn't be used
  while (propsToVisit.length > 0) {
    const key = propsToVisit.pop();

    let parent = payload;
    let keyOnParent = key[0];
    for (let i = 1; i < key.length; i++) {
      parent = parent[key[i - 1]];
      keyOnParent = key[i];
    }

    if (
      !propUsageMap.createOnly.includes(keyOnParent) &&
      !propUsageMap.updatable.includes(keyOnParent)
    ) {
      delete parent[keyOnParent];
      continue;
    }

    const prop = parent[keyOnParent];

    if (typeof prop !== "object" || Array.isArray(prop)) {
      continue;
    }

    for (const childKey in _.keys(prop)) {
      propsToVisit.unshift([...key, childKey]);
    }
  }

  return extLib.removeEmpty(payload);
}
