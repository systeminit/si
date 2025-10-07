async function main(component: Input): Promise<Output> {
  const resourceId = component.properties?.si?.resourceId;
  if (!component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Unable to queue a delete action on a component without a resource",
    };
  }
  const delay = (time: number) => {
    return new Promise((res) => {
      setTimeout(res, time);
    });
  };

  let deleteAttempt = 0;
  const baseDelay = 1000;
  const maxDelay = 90000;
  let progressEvent;

  console.log(`Starting delete operation for resourceId: ${resourceId}, region: ${_.get(component, "properties.domain.extra.Region", "")}`);

  // Retry initial delete operation if rate limited
  while (deleteAttempt < 20) {
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

    console.log(`Delete attempt ${deleteAttempt + 1}: AWS CLI exit code: ${child.exitCode}`);

    if (child.exitCode !== 0) {
      const isRateLimited = child.stderr.includes("Throttling") ||
                           child.stderr.includes("TooManyRequests") ||
                           child.stderr.includes("RequestLimitExceeded") ||
                           child.stderr.includes("ThrottlingException");

      if (isRateLimited && deleteAttempt < 19) {
        console.log(`Delete attempt ${deleteAttempt + 1} rate limited, will retry`);
      } else {
        console.error(`Delete attempt ${deleteAttempt + 1} failed:`, child.stderr);
      }

      if (isRateLimited && deleteAttempt < 19) {
        deleteAttempt++;
        const exponentialDelay = Math.min(baseDelay * Math.pow(2, deleteAttempt - 1), maxDelay);
        const jitter = Math.random() * 0.3 * exponentialDelay;
        const finalDelay = exponentialDelay + jitter;

        console.log(`[DELETE] Rate limited on attempt ${deleteAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
        await delay(finalDelay);
        continue;
      } else {
        return {
          status: "error",
          message:
            `Unable to delete resource; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
      }
    } else {
      console.log(`[DELETE] Initial delete successful on attempt ${deleteAttempt + 1}`);
      progressEvent = JSON.parse(child.stdout);
      console.log(`[DELETE] Got progress event:`, JSON.stringify(progressEvent, null, 2));
      break;
    }
  }

  console.log(`[DELETE] Starting status polling for request token: ${_.get(progressEvent, ["ProgressEvent", "RequestToken"])}`);

  let finished = false;
  let success = false;
  let attempt = 0;
  let message = "";
  let identifier = "";

  while (!finished) {
    console.log(`[DELETE] Status poll attempt ${attempt + 1}`);
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
    console.log(`[DELETE] Status poll ${attempt + 1}: exit code ${child.exitCode}, stderr: ${child.stderr ? 'present' : 'none'}`);

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
        console.log(`[DELETE] Status poll ${attempt + 1} rate limited, will retry`);
        shouldRetry = true;
      } else {
        console.error(`[DELETE] Status poll ${attempt + 1} failed:`, child.stderr);
      }

      if (!isRateLimited || attempt >= 20) {
        return {
          status: "error",
          message:
            `Unable to delete; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
      }
    } else {
      try {
        const currentProgressEvent = JSON.parse(child.stdout);
        console.log(`[DELETE] Status poll ${attempt + 1} response:`, JSON.stringify(currentProgressEvent, null, 2));

        // Log stderr warnings but don't fail if we have valid JSON
        if (hasStderrError) {
          console.warn("AWS CLI stderr (non-fatal):", child.stderr);
        }

        const operationStatus =
          currentProgressEvent["ProgressEvent"]["OperationStatus"];
        if (operationStatus == "SUCCESS") {
          console.log(`[DELETE] Operation SUCCESS detected!`);
          finished = true;
          success = true;
          identifier = currentProgressEvent["ProgressEvent"]["Identifier"];
        } else if (operationStatus == "FAILED") {
          console.log(`[DELETE] Operation FAILED: ${currentProgressEvent["ProgressEvent"]["StatusMessage"] || currentProgressEvent["ProgressEvent"]["ErrorCode"]}`);
          finished = true;
          success = false;
          message = currentProgressEvent["ProgressEvent"]["StatusMessage"] ||
            currentProgressEvent["ProgressEvent"]["ErrorCode"];
        } else if (operationStatus == "CANCEL_COMPLETE") {
          console.log(`[DELETE] Operation CANCELLED`);
          finished = true;
          success = false;
          message = "Operation Canceled by API or AWS.";
        }
      } catch (parseError) {
        console.error("Failed to parse AWS response:", parseError);
        console.error("Raw stdout:", child.stdout);
        console.error("Raw stderr:", child.stderr);

        if (isRateLimited && attempt < 20) {
          console.log(`[DELETE] Parse error with rate limiting on attempt ${attempt + 1}, will retry after backoff`);
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

      console.log(`[DELETE] Waiting ${Math.round(finalDelay)}ms before status poll attempt ${attempt + 1}`);
      await delay(finalDelay);
    }
  }

  console.log(`[DELETE] Final result: success=${success}`);

  if (success) {
    console.log(`[DELETE] Returning success`);
    return {
      payload: null,
      status: "ok",
    };
  } else {
    console.log(`[DELETE] Returning error: ${message}`);
    return {
      message,
      payload: null,
      status: "error",
    };
  }
}
