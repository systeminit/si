async function main(component: Input): Promise<Output> {
  interface CloudControlPayload {
    TypeName: string;
    DesiredState: Record<string, any>;
  }

  const cloudControlType = _.get(component, [
    "domain",
    "extra",
    "AwsResourceType",
  ]);

  const propUsageMap = JSON.parse(component.domain.extra.PropUsageMap);
  if (
    !Array.isArray(propUsageMap.createOnly) ||
    !Array.isArray(propUsageMap.updatable) ||
    !Array.isArray(propUsageMap.secrets)
  ) {
    throw Error("malformed propUsageMap on resource");
  }

  const payload = _.cloneDeep(component.domain);

  // Transform Rules array items from JSON strings to objects
  if (payload.Rules && Array.isArray(payload.Rules)) {
    payload.Rules = payload.Rules.map((rule: any, index: number) => {
      if (typeof rule === 'string') {
        try {
          return JSON.parse(rule);
        } catch (error) {
          console.error(`Error parsing Rules[${index}]:`, error);
          console.error(`Rule value:`, rule);
          // Return the string as-is if parsing fails
          return rule;
        }
      }
      return rule;
    });
  }

  addSecretsToPayload(payload, propUsageMap);

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

  const cleaned = extLib.removeEmpty(payload);

  const cloudControlPayload: CloudControlPayload = {
    TypeName: cloudControlType,
    DesiredState: cleaned,
  };

  return {
    format: "json",
    code: JSON.stringify(cloudControlPayload, null, 2),
  };
}

// If you change this, you should change the same func on awsCloudControlCodeGenUpdate.ts in this same directory
function addSecretsToPayload(
  payload: Record<string, any>,
  propUsageMap: {
    secrets: {
      secretKey: string;
      propPath: string[];
    }[];
  },
) {
  for (
    const {
      secretKey,
      propPath,
    } of propUsageMap.secrets
  ) {
    const secret = requestStorage.getItem(secretKey);

    if (!propPath?.length || propPath.length < 1) {
      throw Error("malformed secret on propUsageMap: bad propPath");
    }
    if (secret) {
      let secretParent = payload;
      let propKey = propPath[0];
      for (let i = 1; i < propPath.length; i++) {
        // Ensure key exists on payload
        secretParent[propKey] = secretParent[propKey] ?? {};
        secretParent = secretParent[propKey];
        propKey = propPath[i];
      }

      secretParent[propKey] = secret;
    }
  }
}
