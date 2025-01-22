async function main({
  thisComponent,
}: Input): Promise<Output> {
  const component = thisComponent.properties;

  let resourceId = _.get(component, ["si", "resourceId"]);

  if (!resourceId) {
    return {
      status: "error",
      message: "No resourceId set, cannot import resource",
    };
  }

  const region = _.get(component, ["domain", "extra", "Region"], "");
  const awsResourceType = _.get(component, [
    "domain",
    "extra",
    "AwsResourceType",
  ], "");

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
    // FIXME: should track down what happens when the resource doesnt exist
    //if (child.stderr.includes("ResourceNotFoundException")) {
    //    console.log("EKS Cluster not found  upstream (ResourceNotFoundException) so removing the resource")
    //    return {
    //        status: "ok",
    //        payload: null,
    //    };
    //}
    return {
      status: "error",
      message:
        `Import error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
    };
  }

  const resourceResponse = JSON.parse(child.stdout);
  const resourceProperties = JSON.parse(
    resourceResponse["ResourceDescription"]["Properties"],
  );
  console.log(resourceProperties);

  let properties = {};
  for (const rprop of Object.keys(resourceProperties)) {
    const category = _.get(component, [
      "domain",
      "extra",
      "AwsFieldMap",
      rprop,
    ]);
    if (category) {
      _.set(
        properties,
        ["domain", category, rprop],
        _.get(resourceProperties, rprop),
      );
    }
  }
  console.log(properties);
  return {
    status: "ok",
    message: "Imported Resource",
    ops: {
      update: {
        self: {
          properties,
        },
      },
      actions: {
        self: {
          remove: ["create"],
          add: ["refresh"],
        },
      },
    },
  };
}
