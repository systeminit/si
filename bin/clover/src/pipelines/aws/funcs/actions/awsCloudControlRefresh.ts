async function main(component: Input): Promise<Output> {
  let name = component.properties?.si?.resourceId;
  const resource = component.properties.resource?.payload;
  if (!name) {
    name = resource.name;
  }
  if (!name) {
    return {
      status: component.properties.resource?.status ?? "error",
      message: "Could not refresh, no resourceId present",
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "cloudcontrol",
    "get-resource",
    "--region",
    _.get(component, "properties.domain.extra.Region", ""),
    "--type-name",
    _.get(component, "properties.domain.extra.AwsResourceType", ""),
    "--identifier",
    name,
  ]);

  if (child.exitCode !== 0) {
    console.log("Failed to refresh cloud control resource");
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
      payload: resource,
      message:
        `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
    };
  }

  const resourceResponse = JSON.parse(child.stdout);
  const payload = JSON.parse(
    resourceResponse["ResourceDescription"]["Properties"],
  );
  return {
    payload,
    status: "ok",
  };
}
