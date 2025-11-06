type Input = {
  domain: {
    extra: {
      AwsResourceType: string;
      PropUsageMap: string;
    };
    [key: string]: any;
  };
};

type Output = {
  format: string;
  code: string;
};

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

// Simple implementation of extLib.removeEmpty for testing
const extLib = {
  removeEmpty(obj: any): any {
    if (Array.isArray(obj)) {
      return obj.map((v) => extLib.removeEmpty(v));
    }
    if (typeof obj === "object" && obj !== null) {
      return Object.entries(obj)
        .filter(([_, v]) =>
          v !== null && v !== undefined && v !== "" &&
          (!Array.isArray(v) || v.length > 0) &&
          (typeof v !== "object" || Object.keys(v).length > 0)
        )
        .reduce((acc, [k, v]) => ({ ...acc, [k]: extLib.removeEmpty(v) }), {});
    }
    return obj;
  },
};

export default main;
