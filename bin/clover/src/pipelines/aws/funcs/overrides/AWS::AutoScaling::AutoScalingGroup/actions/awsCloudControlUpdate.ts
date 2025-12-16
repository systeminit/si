async function main(component: Input): Promise<Output> {
  if (!component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource must exist to be updated",
      payload: component.properties.resource.payload,
    };
  }

  let resourceId = component.properties?.si?.resourceId;

  const delay = (time: number) => {
    return new Promise((res) => {
      setTimeout(res, time);
    });
  };

  const baseDelay = 1000;
  const maxDelay = 90000;
  let refreshAttempt = 0;
  let resourceResponse;

  console.log(`Starting AutoScaling Group update operation - initial refresh for resourceId: ${resourceId}, region: ${_.get(component, "properties.domain.extra.Region", "")}`);

  // Retry initial refresh operation if rate limited
  while (refreshAttempt < 20) {
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

    console.log(`Initial refresh attempt ${refreshAttempt + 1}: AWS CLI exit code: ${refreshChild.exitCode}`);

    if (refreshChild.exitCode !== 0) {
      const isRateLimited = refreshChild.stderr.includes("Throttling") ||
                           refreshChild.stderr.includes("TooManyRequests") ||
                           refreshChild.stderr.includes("RequestLimitExceeded") ||
                           refreshChild.stderr.includes("ThrottlingException");

      if (isRateLimited && refreshAttempt < 19) {
        console.log(`Initial refresh attempt ${refreshAttempt + 1} rate limited, will retry`);
      } else {
        console.log("Failed to refresh cloud control resource");
        console.log(refreshChild.stdout);
        console.error(`Initial refresh attempt ${refreshAttempt + 1} failed:`, refreshChild.stderr);
      }

      if (isRateLimited && refreshAttempt < 19) {
        refreshAttempt++;
        const exponentialDelay = Math.min(baseDelay * Math.pow(2, refreshAttempt - 1), maxDelay);
        const jitter = Math.random() * 0.3 * exponentialDelay;
        const finalDelay = exponentialDelay + jitter;

        console.log(`[ASG-UPDATE] Initial refresh rate limited on attempt ${refreshAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
        await delay(finalDelay);
        continue;
      } else {
        return {
          status: "error",
          message:
            `Update error while fetching current state; exit code ${refreshChild.exitCode}.\n\nSTDOUT:\n\n${refreshChild.stdout}\n\nSTDERR:\n\n${refreshChild.stderr}`,
        };
      }
    } else {
      console.log(`[ASG-UPDATE] Initial refresh successful on attempt ${refreshAttempt + 1}`);
      resourceResponse = JSON.parse(refreshChild.stdout);
      break;
    }
  }

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

  // Keep patch operations using strings to match AWS CloudControl expectations
  // Convert integer properties in desiredProps to strings if AWS returns strings
  const integerProps = ["MaxSize", "MinSize", "Cooldown", "DesiredCapacity"];
  integerProps.forEach(propName => {
    if (desiredProps[propName] !== undefined) {
      // If AWS returns string and desired is number, convert desired to string for patch
      if (typeof currentState[propName] === "string" && typeof desiredProps[propName] === "number") {
        desiredProps[propName] = String(desiredProps[propName]);
        console.log(`[ASG-UPDATE] Converted desiredProps.${propName} to string "${desiredProps[propName]}" to match AWS format`);
      }
      // If AWS returns string and desired is string, keep as string
      // If both are numbers, keep as numbers
    }
  });

  const desiredState = _.cloneDeep(currentState);
  _.merge(desiredState, desiredProps);
  
  console.log(`[ASG-UPDATE] Debug - patch will use currentState types:`, {
    MaxSize: currentState.MaxSize, 
    MinSize: currentState.MinSize, 
    Cooldown: currentState.Cooldown, 
    DesiredCapacity: currentState.DesiredCapacity
  });
  console.log(`[ASG-UPDATE] Debug - patch will use desiredState types:`, {
    MaxSize: desiredState.MaxSize, 
    MinSize: desiredState.MinSize, 
    Cooldown: desiredState.Cooldown, 
    DesiredCapacity: desiredState.DesiredCapacity
  });
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

  let updateAttempt = 0;
  let progressEvent;

  console.log(`Starting AutoScaling Group update operation for resourceId: ${resourceId}`);

  // Retry update operation if rate limited
  while (updateAttempt < 20) {
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

    console.log(`Update attempt ${updateAttempt + 1}: AWS CLI exit code: ${child.exitCode}`);

    if (child.exitCode !== 0) {
      const isRateLimited = child.stderr.includes("Throttling") ||
                           child.stderr.includes("TooManyRequests") ||
                           child.stderr.includes("RequestLimitExceeded") ||
                           child.stderr.includes("ThrottlingException");

      if (isRateLimited && updateAttempt < 19) {
        console.log(`Update attempt ${updateAttempt + 1} rate limited, will retry`);
      } else {
        console.error(`Update attempt ${updateAttempt + 1} failed:`, child.stderr);
      }

      if (isRateLimited && updateAttempt < 19) {
        updateAttempt++;
        const exponentialDelay = Math.min(baseDelay * Math.pow(2, updateAttempt - 1), maxDelay);
        const jitter = Math.random() * 0.3 * exponentialDelay;
        const finalDelay = exponentialDelay + jitter;

        console.log(`[ASG-UPDATE] Update rate limited on attempt ${updateAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
        await delay(finalDelay);
        continue;
      } else {
        return {
          status: "error",
          message:
            `Unable to update; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
      }
    } else {
      console.log(`[ASG-UPDATE] Update successful on attempt ${updateAttempt + 1}`);
      progressEvent = JSON.parse(child.stdout);
      console.log(`[ASG-UPDATE] Got progress event:`, JSON.stringify(progressEvent, null, 2));
      break;
    }
  }

  console.log(`[ASG-UPDATE] Starting status polling for request token: ${_.get(progressEvent, ["ProgressEvent", "RequestToken"])}`);

  let finished = false;
  let success = false;
  let attempt = 0;
  let message = "";
  let identifier = "";

  while (!finished) {
    console.log(`[ASG-UPDATE] Status poll attempt ${attempt + 1}`);
    const child = await siExec.waitUntilEnd("aws", [
      "cloudcontrol",
      "get-resource-request-status",
      "--region",
      _.get(component, "properties.domain.extra.Region", ""),
      "--request-token",
      _.get(progressEvent, ["ProgressEvent", "RequestToken"]),
      "--no-cli-pager",
    ]);

    let shouldRetry = false;
    console.log(`[ASG-UPDATE] Status poll ${attempt + 1}: exit code ${child.exitCode}, stderr: ${child.stderr ? 'present' : 'none'}`);

    // Check for rate limiting in stderr regardless of exit code
    const hasStderrError = child.stderr && child.stderr.trim().length > 0;
    const isRateLimited = hasStderrError && (
      child.stderr.includes("Throttling") ||
      child.stderr.includes("TooManyRequests") ||
      child.stderr.includes("RequestLimitExceeded") ||
      child.stderr.includes("ThrottlingException")
    );

    if (child.exitCode !== 0) {
      if (isRateLimited && attempt < 20) {
        console.log(`[ASG-UPDATE] Status poll ${attempt + 1} rate limited, will retry`);
        shouldRetry = true;
      } else {
        console.error(`[ASG-UPDATE] Status poll ${attempt + 1} failed:`, child.stderr);
      }

      if (!isRateLimited || attempt >= 20) {
        return {
          status: "error",
          message:
            `Unable to update; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
      }
    } else {
      try {
        const currentProgressEvent = JSON.parse(child.stdout);
        console.log(`[ASG-UPDATE] Status poll ${attempt + 1} response:`, JSON.stringify(currentProgressEvent, null, 2));

        // Log stderr warnings but don't fail if we have valid JSON
        if (hasStderrError) {
          console.warn("AWS CLI stderr (non-fatal):", child.stderr);
        }

        const operationStatus =
          currentProgressEvent["ProgressEvent"]["OperationStatus"];
        if (operationStatus == "SUCCESS") {
          console.log(`[ASG-UPDATE] Operation SUCCESS detected! Resource ID: ${currentProgressEvent["ProgressEvent"]["Identifier"]}`);
          finished = true;
          success = true;
          identifier = currentProgressEvent["ProgressEvent"]["Identifier"];
        } else if (operationStatus == "FAILED") {
          console.log(`[ASG-UPDATE] Operation FAILED: ${currentProgressEvent["ProgressEvent"]["StatusMessage"] || currentProgressEvent["ProgressEvent"]["ErrorCode"]}`);
          finished = true;
          success = false;
          message = currentProgressEvent["ProgressEvent"]["StatusMessage"] ||
            currentProgressEvent["ProgressEvent"]["ErrorCode"];
        } else if (operationStatus == "CANCEL_COMPLETE") {
          console.log(`[ASG-UPDATE] Operation CANCELLED`);
          finished = true;
          success = false;
          message = "Operation Canceled by API or AWS.";
        }
      } catch (parseError) {
        console.error("Failed to parse AWS response:", parseError);
        console.error("Raw stdout:", child.stdout);
        console.error("Raw stderr:", child.stderr);

        if (isRateLimited && attempt < 20) {
          console.log(`[ASG-UPDATE] Parse error with rate limiting on attempt ${attempt + 1}, will retry after backoff`);
          shouldRetry = true;
        } else {
          return {
            status: "error",
            message: "Unable to parse AWS CloudControl response",
          };
        }
      }
    }

    if (!finished || shouldRetry) {
      attempt++;
      const exponentialDelay = Math.min(baseDelay * Math.pow(2, attempt - 1), maxDelay);
      const jitter = Math.random() * 0.3 * exponentialDelay;
      const finalDelay = exponentialDelay + jitter;

      console.log(`[ASG-UPDATE] Waiting ${Math.round(finalDelay)}ms before status poll attempt ${attempt + 1}`);
      await delay(finalDelay);
    }
  }

  console.log(`[ASG-UPDATE] Final result: success=${success}, identifier=${identifier}`);

  if (success) {
    console.log(`[ASG-UPDATE] Starting final refresh for updated AutoScaling Group`);
    let finalRefreshAttempt = 0;
    let finalResourceResponse;

    // Retry final refresh operation if rate limited
    while (finalRefreshAttempt < 20) {
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

      console.log(`Final refresh attempt ${finalRefreshAttempt + 1}: AWS CLI exit code: ${child.exitCode}`);

      if (child.exitCode !== 0) {
        const isRateLimited = child.stderr.includes("Throttling") ||
                             child.stderr.includes("TooManyRequests") ||
                             child.stderr.includes("RequestLimitExceeded") ||
                             child.stderr.includes("ThrottlingException");

        if (isRateLimited && finalRefreshAttempt < 19) {
          console.log(`Final refresh attempt ${finalRefreshAttempt + 1} rate limited, will retry`);
        } else {
          console.log("Failed to refresh cloud control resource");
          console.log(child.stdout);
          console.error(`Final refresh attempt ${finalRefreshAttempt + 1} failed:`, child.stderr);
        }

        if (isRateLimited && finalRefreshAttempt < 19) {
          finalRefreshAttempt++;
          const exponentialDelay = Math.min(baseDelay * Math.pow(2, finalRefreshAttempt - 1), maxDelay);
          const jitter = Math.random() * 0.3 * exponentialDelay;
          const finalDelay = exponentialDelay + jitter;

          console.log(`[ASG-UPDATE] Final refresh rate limited on attempt ${finalRefreshAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
          await delay(finalDelay);
          continue;
        } else {
          return {
            status: "error",
            payload: _.get(component, "properties.resource.payload"),
            message:
              `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
          };
        }
      } else {
        console.log(`[ASG-UPDATE] Final refresh successful on attempt ${finalRefreshAttempt + 1}`);
        finalResourceResponse = JSON.parse(child.stdout);
        break;
      }
    }

    const payload = JSON.parse(
      finalResourceResponse["ResourceDescription"]["Properties"],
    );
    
    // Convert string properties that should be integers to numbers for final payload
    const integerProps = ["MaxSize", "MinSize", "Cooldown", "DesiredCapacity"];
    integerProps.forEach(propName => {
      if (payload[propName] && typeof payload[propName] === "string") {
        const numValue = parseInt(payload[propName], 10);
        if (!isNaN(numValue)) {
          payload[propName] = numValue;
          console.log(`[ASG-UPDATE] Converted final payload.${propName} from "${payload[propName]}" to ${numValue}`);
        }
      }
    });
    
    console.log(`[ASG-UPDATE] Returning success with updated AutoScaling Group payload`);
    return {
      payload,
      status: "ok",
    };
  } else {
    console.log(`[ASG-UPDATE] Returning error: ${message}`);
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
