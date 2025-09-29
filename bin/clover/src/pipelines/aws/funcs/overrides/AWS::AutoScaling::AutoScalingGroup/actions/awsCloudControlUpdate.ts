async function main(component: Input): Promise<Output> {
  if (!component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource must exist to be updated",
      payload: component.properties.resource.payload,
    };
  }

  let resourceId = component.properties?.si?.resourceId;

  const refreshChild = await siExec.waitUntilEnd("aws", [
    "cloudcontrol",
    "get-resource",
    "--region",
    _.get(component, "properties.domain.extra.Region", ""),
    "--type-name",
    _.get(component, "properties.domain.extra.AwsResourceType", ""),
    "--identifier",
    resourceId,
  ]);

  if (refreshChild.exitCode !== 0) {
    console.log("Failed to refresh cloud control resource");
    console.log(refreshChild.stdout);
    console.error(refreshChild.stderr);
    return {
      status: "error",
      message:
        `Update error while fetching current state; exit code ${refreshChild.exitCode}.\n\nSTDOUT:\n\n${refreshChild.stdout}\n\nSTDERR:\n\n${refreshChild.stderr}`,
    };
  }

  const resourceResponse = JSON.parse(refreshChild.stdout);
  const currentState = JSON.parse(
    resourceResponse["ResourceDescription"]["Properties"],
  );

  const desiredProps = JSON.parse(
    component.properties.code?.["awsCloudControlUpdate"]?.code,
  )?.DesiredState;

  // Copy secrets to desired props
  const propUsageMap = JSON.parse(
    component.properties?.domain.extra.PropUsageMap,
  );

  addSecretsToPayload(desiredProps, propUsageMap);

  const desiredState = _.cloneDeep(currentState);
  _.merge(desiredState, desiredProps);
  let patch;
  try {
    patch = jsonpatch.compare(currentState, desiredState, true);

    // Fix for LaunchTemplate updates - ensure complete object is sent
    // Only apply this fix when updating an existing LaunchTemplate's version
    const hasLaunchTemplateVersionPatch = patch.some((op) =>
      op.path === "/LaunchTemplate/Version" &&
      (op.op === "replace" || op.op === "add")
    );

    if (
      hasLaunchTemplateVersionPatch && currentState.LaunchTemplate &&
      desiredState.LaunchTemplate
    ) {
      // Remove any partial LaunchTemplate patches
      patch = patch.filter((op) => !op.path.startsWith("/LaunchTemplate"));

      // Only include LaunchTemplateId and Version for the update
      patch.push({
        op: "replace",
        path: "/LaunchTemplate",
        value: {
          LaunchTemplateId: desiredState.LaunchTemplate.LaunchTemplateId,
          Version: desiredState.LaunchTemplate.Version,
        },
      });
    }
  } catch (e) {
    return {
      status: "error",
      message: `jsonpatch error\n\nMessage: ${e}`,
    };
  }
  console.log("Computed patch", patch);

  const child = await siExec.waitUntilEnd("aws", [
    "cloudcontrol",
    "update-resource",
    "--region",
    _.get(component, "properties.domain.extra.Region", ""),
    "--type-name",
    _.get(component, "properties.domain.extra.AwsResourceType", ""),
    "--identifier",
    resourceId,
    "--patch-document",
    JSON.stringify(patch),
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message:
        `Unable to update; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }

  const progressEvent = JSON.parse(child.stdout);
  console.log("Progress Event", progressEvent);

  const delay = (time: number) => {
    return new Promise((res) => {
      setTimeout(res, time);
    });
  };

  let finished = false;
  let success = false;
  let wait = 1000;
  const upperLimit = 10000;
  let message = "";
  let identifier = "";

  while (!finished) {
    const child = await siExec.waitUntilEnd("aws", [
      "cloudcontrol",
      "get-resource-request-status",
      "--region",
      _.get(component, "properties.domain.extra.Region", ""),
      "--request-token",
      _.get(progressEvent, ["ProgressEvent", "RequestToken"]),
    ]);

    if (child.exitCode !== 0) {
      console.error(child.stderr);
      return {
        status: "error",
        message:
          `Unable to update; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
      };
    }
    const currentProgressEvent = JSON.parse(child.stdout);
    console.log("Current Progress", currentProgressEvent);
    const operationStatus =
      currentProgressEvent["ProgressEvent"]["OperationStatus"];
    if (operationStatus == "SUCCESS") {
      finished = true;
      success = true;
      identifier = currentProgressEvent["ProgressEvent"]["Identifier"];
    } else if (operationStatus == "FAILED") {
      finished = true;
      success = false;
      message = currentProgressEvent["ProgressEvent"]["StatusMessage"] ||
        currentProgressEvent["ProgressEvent"]["ErrorCode"];
    } else if (operationStatus == "CANCEL_COMPLETE") {
      finished = true;
      success = false;
      message = "Operation Canceled by API or AWS.";
    }

    if (!finished) {
      console.log("\nWaiting to check status!\n");
      await delay(wait);
      if (wait != upperLimit) {
        wait = wait + 1000;
      }
    }
  }

  if (success) {
    const child = await siExec.waitUntilEnd("aws", [
      "cloudcontrol",
      "get-resource",
      "--region",
      _.get(component, "properties.domain.extra.Region", ""),
      "--type-name",
      _.get(component, "properties.domain.extra.AwsResourceType", ""),
      "--identifier",
      identifier,
    ]);

    if (child.exitCode !== 0) {
      console.log("Failed to refresh cloud control resource");
      console.log(child.stdout);
      console.error(child.stderr);
      return {
        status: "error",
        payload: _.get(component, "properties.resource.payload"),
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
  } else {
    return {
      message,
      payload: _.get(component, "properties.resource.payload"),
      status: "error",
    };
  }
}

// If you change this, you should change the same func on awsCloudControlCreate.ts in this same directory
function addSecretsToPayload(
  payload: Record<string, any>,
  propUsageMap: {
    secrets: {
      secretKey: string;
      propPath: string[];
    }[];
  },
) {
  if (
    !Array.isArray(propUsageMap.secrets)
  ) {
    throw Error("malformed propUsageMap on asset");
  }

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
    if (!secret) continue;

    let secretParent = payload;
    let propKey = propPath[0];
    for (let i = 1; i < propPath.length; i++) {
      const thisProp = secretParent[propKey];

      if (!thisProp) {
        break;
      }

      secretParent = secretParent[propKey];
      propKey = propPath[i];
    }

    // Only add secret to payload if the codegen output has it
    if (propKey in secretParent) {
      secretParent[propKey] = secret;
    }
  }
}
