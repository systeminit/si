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

  const delay = (time: number) => {
    return new Promise((res) => {
      setTimeout(res, time);
    });
  };

  let refreshAttempt = 0;
  const baseDelay = 1000;
  const maxDelay = 90000;
  let resourceResponse;

  console.log(`Starting refresh operation for resourceId: ${name}, region: ${_.get(component, "properties.domain.extra.Region", "")}`);

  // Retry refresh operation if rate limited
  while (refreshAttempt < 20) {
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

    console.log(`Refresh attempt ${refreshAttempt + 1}: AWS CLI exit code: ${child.exitCode}`);

    if (child.exitCode !== 0) {
      const isRateLimited = child.stderr.includes("Throttling") ||
                           child.stderr.includes("TooManyRequests") ||
                           child.stderr.includes("RequestLimitExceeded") ||
                           child.stderr.includes("ThrottlingException");

      // Check if this is a ResourceNotFoundException (valid case for deleted resources)
      const isResourceNotFound = child.stderr.includes("ResourceNotFoundException");

      if (isRateLimited && refreshAttempt < 19) {
        console.log(`Refresh attempt ${refreshAttempt + 1} rate limited, will retry`);
      } else if (isResourceNotFound) {
        console.log("Resource not found upstream, will handle as deletion");
      } else {
        console.log("Failed to refresh cloud control resource");
        console.log(child.stdout);
        console.error(`Refresh attempt ${refreshAttempt + 1} failed:`, child.stderr);
      }

      if (isRateLimited && refreshAttempt < 19) {
        refreshAttempt++;
        const exponentialDelay = Math.min(baseDelay * Math.pow(2, refreshAttempt - 1), maxDelay);
        const jitter = Math.random() * 0.3 * exponentialDelay;
        const finalDelay = exponentialDelay + jitter;

        console.log(`[REFRESH] Rate limited on attempt ${refreshAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
        await delay(finalDelay);
        continue;
      } else {
        if (child.stderr.includes("ResourceNotFoundException")) {
           console.log("Component not found upstream (ResourceNotFoundException) so removing the resource")
           return {
               status: "ok",
               payload: null,
           };
        } else {
          return {
            status: "error",
            payload: resource,
            message:
              `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
          };
        }
      }
    } else {
      console.log(`[REFRESH] Refresh successful on attempt ${refreshAttempt + 1}`);
      resourceResponse = JSON.parse(child.stdout);
      break;
    }
  }

  console.log(`[REFRESH] Final result: success, parsing resource response`);
  const payload = JSON.parse(
    resourceResponse["ResourceDescription"]["Properties"],
  );
  return {
    payload,
    status: "ok",
  };
}
