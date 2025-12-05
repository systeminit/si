async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent.properties;
  const region = _.get(component, ["domain", "extra", "Region"]);
  const awsResourceType = _.get(
    component,
    ["domain", "extra", "AwsResourceType"],
  );

  if (!region) {
    return {
      status: "error",
      message: "No region found for this resource",
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "cloudcontrol",
    "list-resources",
    "--region",
    region,
    "--type-name",
    awsResourceType,
  ]);

  if (child.exitCode !== 0) {
    console.log("Failed to discover cloud control resources");
    console.log(child.stdout);
    console.error(child.stderr);
    return {
      status: "error",
      message: `Discover error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
    };
  }

  const resourceResponse = JSON.parse(child.stdout);
  const resources = resourceResponse["ResourceDescriptions"] || [];

  console.log(`Discovered ${resources.length} ${awsResourceType} resources`);

  const create = {};
  const actions = {};
  let importCount = 0;

  for (const resource of resources) {
    const resourceId = resource["Identifier"];
    const resourceProperties = JSON.parse(resource["Properties"]);
    
    // Convert string properties that should be integers to numbers
    const integerProps = ["MaxSize", "MinSize", "Cooldown", "DesiredCapacity"];
    integerProps.forEach(propName => {
      if (resourceProperties[propName] && typeof resourceProperties[propName] === "string") {
        const numValue = parseInt(resourceProperties[propName], 10);
        if (!isNaN(numValue)) {
          resourceProperties[propName] = numValue;
          console.log(`Converted ${propName} from "${resourceProperties[propName]}" to ${numValue}`);
        }
      }
    });

    console.log(`Processing resource: ${resourceId}`);

    const properties = {
      si: {
        resourceId,
      },
      domain: {
        extra: {
          Region: region,
        },
        ...resourceProperties,
      },
      resource: resourceProperties,
    };

    create[resourceId] = {
      kind: awsResourceType,
      properties,
    };
    
    actions[resourceId] = {
      remove: ["create"],
    };
    
    importCount += 1;
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} resources`,
    ops: {
      create,
      actions,
    },
  };
}