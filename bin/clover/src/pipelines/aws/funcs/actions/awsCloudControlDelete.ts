async function main(component: Input): Promise<Output> {
  const resourceId = component.properties?.si?.resourceId;
  if (!component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Unable to queue a delete action on a component without a resource",
    };
  }
  const child = await siExec.waitUntilEnd("aws", [
    "cloudcontrol",
    "delete-resource",
    "--region",
    _.get(component, "properties.domain.extra.Region", ""),
    "--type-name",
    _.get(component, "properties.domain.extra.AwsResourceType", ""),
    "--identifier",
    resourceId,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message:
        `Unable to delete resource; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
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
          `Unable to create; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
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
    return {
      payload: null,
      status: "ok",
    };
  } else {
    return {
      message,
      payload: null,
      status: "error",
    };
  }
}
