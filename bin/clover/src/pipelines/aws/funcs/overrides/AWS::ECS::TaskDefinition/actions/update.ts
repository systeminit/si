async function main(component: Input): Promise<Output> {
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

  const delay = (time: number) => {
    return new Promise((res) => {
      setTimeout(res, time);
    });
  };

  let createAttempt = 0;
  const baseDelay = 1000;
  const maxDelay = 120000;
  let progressEvent;

  console.log(`Starting ECS TaskDefinition update (create) operation for resource type: ${code["TypeName"]}, region: ${domain?.extra?.Region}`);

  // Retry initial create operation if rate limited
  while (createAttempt < 10) {
    const child = await siExec.waitUntilEnd("aws", [
      "cloudcontrol",
      "create-resource",
      "--region",
      domain?.extra?.Region || "",
      "--cli-input-json",
      inputJson || "",
    ]);

    console.log(`Create attempt ${createAttempt + 1}: AWS CLI exit code: ${child.exitCode}`);

    if (child.exitCode !== 0) {
      const isRateLimited = child.stderr.includes("Throttling") || 
                           child.stderr.includes("TooManyRequests") ||
                           child.stderr.includes("RequestLimitExceeded") ||
                           child.stderr.includes("ThrottlingException");
      
      if (isRateLimited && createAttempt < 9) {
        console.log(`Create attempt ${createAttempt + 1} rate limited, will retry`);
      } else {
        console.error(`Create attempt ${createAttempt + 1} failed:`, child.stderr);
      }
      
      if (isRateLimited && createAttempt < 9) {
        createAttempt++;
        const exponentialDelay = Math.min(baseDelay * Math.pow(2, createAttempt - 1), maxDelay);
        const jitter = Math.random() * 0.3 * exponentialDelay;
        const finalDelay = exponentialDelay + jitter;
        
        console.log(`[ECS-TASKDEF-UPDATE] Create rate limited on attempt ${createAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
        await delay(finalDelay);
        continue;
      } else {
        return {
          status: "error",
          message:
            `Unable to create; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
      }
    } else {
      console.log(`[ECS-TASKDEF-UPDATE] Initial create successful on attempt ${createAttempt + 1}`);
      progressEvent = JSON.parse(child.stdout);
      console.log(`[ECS-TASKDEF-UPDATE] Got progress event:`, JSON.stringify(progressEvent, null, 2));
      break;
    }
  }

  console.log(`[ECS-TASKDEF-UPDATE] Starting status polling for request token: ${_.get(progressEvent, ["ProgressEvent", "RequestToken"])}`);

  let finished = false;
  let success = false;
  let attempt = 0;
  let message = "";
  let identifier = "";

  while (!finished) {
    console.log(`[ECS-TASKDEF-UPDATE] Status poll attempt ${attempt + 1}`);
    const child = await siExec.waitUntilEnd("aws", [
      "cloudcontrol",
      "get-resource-request-status",
      "--region",
      domain?.extra?.Region || "",
      "--request-token",
      _.get(progressEvent, ["ProgressEvent", "RequestToken"]),
      "--no-cli-pager",
    ]);

    let shouldRetry = false;
    console.log(`[ECS-TASKDEF-UPDATE] Status poll ${attempt + 1}: exit code ${child.exitCode}, stderr: ${child.stderr ? 'present' : 'none'}`);

    // Check for rate limiting in stderr regardless of exit code
    const hasStderrError = child.stderr && child.stderr.trim().length > 0;
    const isRateLimited = hasStderrError && (
      child.stderr.includes("Throttling") || 
      child.stderr.includes("TooManyRequests") ||
      child.stderr.includes("RequestLimitExceeded") ||
      child.stderr.includes("ThrottlingException")
    );

    if (child.exitCode !== 0) {
      if (isRateLimited && attempt < 10) {
        console.log(`[ECS-TASKDEF-UPDATE] Status poll ${attempt + 1} rate limited, will retry`);
        shouldRetry = true;
      } else {
        console.error(`[ECS-TASKDEF-UPDATE] Status poll ${attempt + 1} failed:`, child.stderr);
      }
      
      if (!isRateLimited || attempt >= 10) {
        return {
          status: "error",
          message:
            `Unable to create; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
      }
    } else {
      try {
        const currentProgressEvent = JSON.parse(child.stdout);
        console.log(`[ECS-TASKDEF-UPDATE] Status poll ${attempt + 1} response:`, JSON.stringify(currentProgressEvent, null, 2));
        
        // Log stderr warnings but don't fail if we have valid JSON
        if (hasStderrError) {
          console.warn("AWS CLI stderr (non-fatal):", child.stderr);
        }
        
        const operationStatus =
          currentProgressEvent["ProgressEvent"]["OperationStatus"];
        if (operationStatus == "SUCCESS") {
          console.log(`[ECS-TASKDEF-UPDATE] Operation SUCCESS detected! Resource ID: ${currentProgressEvent["ProgressEvent"]["Identifier"]}`);
          finished = true;
          success = true;
          identifier = currentProgressEvent["ProgressEvent"]["Identifier"];
        } else if (operationStatus == "FAILED") {
          console.log(`[ECS-TASKDEF-UPDATE] Operation FAILED: ${currentProgressEvent["ProgressEvent"]["StatusMessage"] || currentProgressEvent["ProgressEvent"]["ErrorCode"]}`);
          finished = true;
          success = false;
          message = currentProgressEvent["ProgressEvent"]["StatusMessage"] ||
            currentProgressEvent["ProgressEvent"]["ErrorCode"];
        } else if (operationStatus == "CANCEL_COMPLETE") {
          console.log(`[ECS-TASKDEF-UPDATE] Operation CANCELLED`);
          finished = true;
          success = false;
          message = "Operation Canceled by API or AWS.";
        }
      } catch (parseError) {
        console.error("Failed to parse AWS response:", parseError);
        console.error("Raw stdout:", child.stdout);
        console.error("Raw stderr:", child.stderr);
        
        if (isRateLimited && attempt < 10) {
          console.log(`[ECS-TASKDEF-UPDATE] Parse error with rate limiting on attempt ${attempt + 1}, will retry after backoff`);
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
      
      console.log(`[ECS-TASKDEF-UPDATE] Waiting ${Math.round(finalDelay)}ms before status poll attempt ${attempt + 1}`);
      await delay(finalDelay);
    }
  }
  console.log(`[ECS-TASKDEF-UPDATE] Final result: success=${success}, identifier=${identifier}`);
  
  if (success) {
    console.log(`[ECS-TASKDEF-UPDATE] Starting final refresh for created TaskDefinition`);
    let finalRefreshAttempt = 0;
    let finalResourceResponse;

    // Retry final refresh operation if rate limited
    while (finalRefreshAttempt < 10) {
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
        
        if (isRateLimited && finalRefreshAttempt < 9) {
          console.log(`Final refresh attempt ${finalRefreshAttempt + 1} rate limited, will retry`);
        } else {
          console.log("Failed to refresh cloud control resource");
          console.log(child.stdout);
          console.error(`Final refresh attempt ${finalRefreshAttempt + 1} failed:`, child.stderr);
        }
        
        if (isRateLimited && finalRefreshAttempt < 9) {
          finalRefreshAttempt++;
          const exponentialDelay = Math.min(baseDelay * Math.pow(2, finalRefreshAttempt - 1), maxDelay);
          const jitter = Math.random() * 0.3 * exponentialDelay;
          const finalDelay = exponentialDelay + jitter;
          
          console.log(`[ECS-TASKDEF-UPDATE] Final refresh rate limited on attempt ${finalRefreshAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
          await delay(finalDelay);
          continue;
        } else {
          return {
            status: "error",
            message:
              `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
          };
        }
      } else {
        console.log(`[ECS-TASKDEF-UPDATE] Final refresh successful on attempt ${finalRefreshAttempt + 1}`);
        finalResourceResponse = JSON.parse(child.stdout);
        break;
      }
    }

    const payload = JSON.parse(
      finalResourceResponse["ResourceDescription"]["Properties"],
    );
    console.log(`[ECS-TASKDEF-UPDATE] Returning success with TaskDefinition payload`);
    return {
      payload,
      status: "ok",
    };
  } else {
    console.log(`[ECS-TASKDEF-UPDATE] Returning error: ${message}`);
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
