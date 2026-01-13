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

  const resources = {};
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
