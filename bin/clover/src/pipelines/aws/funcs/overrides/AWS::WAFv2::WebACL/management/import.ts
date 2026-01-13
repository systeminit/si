async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent.properties;
  const resourcePayload = _.get(component, ["resource", "payload"], "");
  let resourceId = _.get(component, ["si", "resourceId"]);

  if (!resourceId) {
    return {
      status: "error",
      message: "No resourceId set, cannot import resource",
    };
  }

  const region = _.get(component, ["domain", "extra", "Region"], "");
  const awsResourceType = _.get(
    component,
    ["domain", "extra", "AwsResourceType"],
    "",
  );

  const child = await siExec.waitUntilEnd("aws", [
    "cloudcontrol",
    "get-resource",
    "--region",
    region,
    "--type-name",
    awsResourceType,
    "--identifier",
    resourceId,
  ]);

  if (child.exitCode !== 0) {
    console.log("Failed to import cloud control resource");
    console.log(child.stdout);
    console.error(child.stderr);
    return {
      status: "error",
      message: `Import error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
    };
  }

  const resourceResponse = JSON.parse(child.stdout);
  const resourceProperties = JSON.parse(
    resourceResponse["ResourceDescription"]["Properties"],
  );

  // Transform Rules array items from objects to JSON strings
  if (resourceProperties.Rules && Array.isArray(resourceProperties.Rules)) {
    resourceProperties.Rules = resourceProperties.Rules.map((rule: any, index: number) => {
      if (typeof rule === 'object' && rule !== null) {
        try {
          return JSON.stringify(rule);
        } catch (error) {
          console.error(`Error stringifying Rules[${index}]:`, error);
          console.error(`Rule value:`, rule);
          // Return the object as-is if stringification fails
          return rule;
        }
      }
      return rule;
    });
  }

  console.log(resourceProperties);

  const properties = {
    ...component,
    domain: {
      extra: {
        Region: region,
      },
      ...component.domain,
      ...resourceProperties,
    },
  };

  let needsRefresh = true;
  if (!resourcePayload) {
    properties.resource = resourceProperties;
    needsRefresh = false;
  }

  const ops = {
    update: {
      self: {
        properties,
      },
    },
    actions: {
      self: {
        remove: ["create"],
        add: [] as string[],
      },
    },
  };

  if (needsRefresh) {
    ops.actions.self.add.push("refresh");
  } else {
    ops.actions.self.remove.push("refresh");
  }

  return {
    status: "ok",
    message: "Imported Resource",
    ops,
  };
}
