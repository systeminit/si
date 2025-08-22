async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent.properties;
  const region = _.get(component, ["domain", "extra", "Region"], "");
  const awsResourceType = _.get(
    component,
    ["domain", "extra", "AwsResourceType"],
    "",
  );

  let resourceList = [];
  let finished = false;
  let nextToken = "";
  const create: Output["ops"]["create"] = {};
  const actions = {};

  const refinement = _.cloneDeep(thisComponent.properties.domain);
  // Remove the 'extra' tree, as it obviously isn't 1:1
  delete refinement["extra"];
  // Remove any empty values, as they are never refinements
  for (const [key, value] of Object.entries(refinement)) {
    if (_.isEmpty(value)) {
      delete refinement[key];
    } else if (_.isPlainObject(value)) {
      refinement[key] = _.pickBy(
        value,
        (v) => !_.isEmpty(v) || _.isNumber(v) || _.isBoolean(v),
      );
      if (_.isEmpty(refinement[key])) {
        delete refinement[key];
      }
    }
  }

  while (!finished) {
    const listArgs = [
      "cloudcontrol",
      "list-resources",
      "--region",
      region,
      "--type-name",
      awsResourceType,
    ];
    if (!_.isEmpty(refinement)) {
      listArgs.push("--resource-model");
      listArgs.push(JSON.stringify(refinement));
    }
    if (nextToken) {
      listArgs.push(nextToken);
    }

    const listChild = await siExec.waitUntilEnd("aws", listArgs);

    if (listChild.exitCode !== 0) {
      console.log("Failed to list cloud control resources");
      console.log(listChild.stdout);
      console.error(listChild.stderr);
      return {
        status: "error",
        message: `Resource list error; exit code ${listChild.exitCode}.\n\nSTDOUT:\n\n${listChild.stdout}\n\nSTDERR:\n\n${listChild.stderr}`,
      };
    }
    const listResponse = JSON.parse(listChild.stdout);
    if (listResponse["NextToken"]) {
      nextToken = listResponse["NextToken"];
    } else {
      finished = true;
    }
    resourceList = _.union(resourceList, listResponse["ResourceDescriptions"]);
  }

  let importCount = 0;
  for (const resource of resourceList) {
    let resourceId = resource["Identifier"];
    console.log(`Importing ${resourceId}`);

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
      console.log(
        "Failed to import cloud control resource; continuing, as this is often an AWS bug.",
      );
      console.log(child.stdout);
      console.log(child.stderr);
      continue;
    }

    const resourceResponse = JSON.parse(child.stdout);
    const resourceProperties = JSON.parse(
      resourceResponse["ResourceDescription"]["Properties"],
    );

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

    if (_.isEmpty(refinement) || _.isMatch(properties.domain, refinement)) {
      const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
      for (const [skey, svalue] of Object.entries(thisComponent.sources)) {
        newAttributes[skey] = {
          $source: svalue,
        };
      }

      create[resourceId] = {
        kind: awsResourceType,
        properties,
        attributes: newAttributes,
      };
      actions[resourceId] = {
        remove: ["create"],
      };
      importCount += 1;
    } else {
      console.log(
        `Skipping import of ${resourceId}; it did not match refinements`,
      );
    }
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} Components`,
    ops: {
      create,
      actions,
    },
  };
}
