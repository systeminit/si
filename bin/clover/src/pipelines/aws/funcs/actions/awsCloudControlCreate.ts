async function main(component: Input): Promise<Output> {
  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  const codeString = component.properties.code?.["awsCloudControlCreate"]?.code;

  if (!codeString) {
    return {
      status: "error",
      message: `Could not find awsCloudControlCreate for resource`,
    };
  }

  const domain = component.properties?.domain;
  const code = JSON.parse(codeString);

  const payload = code["DesiredState"];
  const propUsageMap = JSON.parse(domain.extra.PropUsageMap);
  addSecretsToPayload(payload, propUsageMap);

  const inputObject = {
    TypeName: code["TypeName"],
    DesiredState: JSON.stringify(payload),
  };
  const inputJson = JSON.stringify(inputObject);

  const child = await siExec.waitUntilEnd("aws", [
    "cloudcontrol",
    "create-resource",
    "--region",
    domain?.extra?.Region || "",
    "--cli-input-json",
    inputJson || "",
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message:
        `Unable to create; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
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
      domain?.extra?.Region || "",
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
      resourceId: identifier,
      status: "ok",
    };
  } else {
    return {
      message,
      status: "error",
    };
  }
}

// If you change this, you should change the same func on awsCloudControlUpdate.ts in this same directory
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
