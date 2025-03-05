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

    console.log(`Visiting ${key}`);

    let parent = payload;
    let keyOnParent = key[0];
    for (let i = 1; i < key.length; i++) {
      parent = parent[key[i - 1]];
      keyOnParent = key[i];
    }

    if (
      !propUsageMap.updatable.includes(keyOnParent)
    ) {
      delete parent[keyOnParent];
      console.log("Removing " + key);
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

  const cleaned = removeEmpty(payload);

  const cloudControlPayload: CloudControlPayload = {
    TypeName: cloudControlType,
    DesiredState: cleaned,
  };

  return {
    format: "json",
    code: JSON.stringify(cloudControlPayload, null, 2),
  };
}

function removeEmpty(obj: any): any {
  // Stack to keep track of objects to process
  const stack: any = [{
    parent: null,
    key: null,
    value: obj,
  }];

  while (stack.length) {
    const {
      parent,
      key,
      value,
    } = stack.pop();

    if (_.isObject(value)) {
      // Iterate over the keys of the current object
      _.forOwn(value, (childValue, childKey) => {
        stack.push({
          parent: value,
          key: childKey,
          value: childValue,
        });
      });

      // After processing children, check if the current object is empty
      if (_.isEmpty(value) && parent) {
        if (_.isArray(parent)) {
          parent.splice(key, 1);
        } else {
          delete parent[key];
        }
      }
    } else if (_.isArray(value)) {
      // Iterate over the array elements
      for (let i = value.length - 1; i >= 0; i--) {
        stack.push({
          parent: value,
          key: i,
          value: value[i],
        });
      }

      // After processing elements, check if the current array is empty
      if (_.isEmpty(value) && parent) {
        parent.splice(key, 1);
      }
    } else {
      // Handle primitive values: remove if null, undefined, or empty string
      if (value === null || value === undefined || value === "") {
        if (_.isArray(parent)) {
          parent.splice(key, 1);
        } else {
          delete parent[key];
        }
      }
    }
  }

  // Final pass to remove any remaining empty objects or arrays at the root level
  _.forOwn(obj, (value, key) => {
    if (
      (_.isObject(value) && _.isEmpty(value)) ||
      value === null ||
      value === undefined ||
      value === ""
    ) {
      delete obj[key];
    }
  });

  return obj;
}

// If you change this, you should change the same func on awsCloudControlCodeGenCreate.ts in this same directory
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
