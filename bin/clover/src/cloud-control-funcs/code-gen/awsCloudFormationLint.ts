async function main(component: Input): Promise<Output> {
  const cloudControlType = _.get(component, [
    "domain",
    "extra",
    "AwsResourceType",
  ]);

  const propUsageMap = JSON.parse(component.domain.extra.PropUsageMap);
  if (
    !Array.isArray(propUsageMap.createOnly) ||
    !Array.isArray(propUsageMap.updatable)
  ) {
    throw Error("malformed propUsageMap on resource");
  }

  const payload = _.cloneDeep(component.domain);

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
      !propUsageMap.createOnly.includes(keyOnParent) &&
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

  const resources = {};
  console.log('foo', component);
  resources["cfnResource"] = {
    Type: cloudControlType,
    Properties: cleaned,
  };
  const cloudFormationPayload = {
    AWSTemplateFormatVersion: "2010-09-09",
    Resources: resources,
  };

  return {
    format: "json",
    code: JSON.stringify(cloudFormationPayload, null, 2),
  };
}

function removeEmpty(obj: any): any {
  if (Array.isArray(obj)) {
    return obj
      .filter((item) => {
        if (item === null || item === undefined || item === "") return false;
        if (Array.isArray(item) && item.length === 0) return false;
        if (typeof item === "object" && Object.keys(item).length === 0) {
          return false;
        }
        return true;
      })
      .map((item) =>
        typeof item === "object" && item !== null ? removeEmpty(item) : item
      );
  }

  return Object.fromEntries(
    Object.entries(obj)
      .filter(([_, value]) => {
        if (value === null || value === undefined || value === "") return false;
        if (Array.isArray(value) && value.length === 0) return false;
        if (typeof value === "object" && Object.keys(value).length === 0) {
          return false;
        }
        return true;
      })
      .map(([key, value]) => [
        key,
        typeof value === "object" && value !== null
          ? removeEmpty(value)
          : value,
      ]),
  );
}
