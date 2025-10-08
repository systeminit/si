async function main(component: Input): Promise<Output> {
  let name = component.properties?.si?.resourceId;
  const resource = component.properties.resource?.payload;

  if (!name) {
    name = resource?.name;
  }

  const output: Output = {
    payload: resource,
    status: component.properties.resource?.status ?? "error",
    message: "Could not refresh, no resourceId present",
  };

  if (!name) {
    return output;
  }

  const delay = (time: number) => {
    return new Promise((res) => {
      setTimeout(res, time);
    });
  };

  const baseDelay = 1000;
  const maxDelay = 90000;
  let refreshAttempt = 0;
  let resourceResponse;

  console.log(`Starting VPNConnection refresh operation for resourceId: ${name}, region: ${_.get(component, "properties.domain.extra.Region", "")}`);

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

      if (isRateLimited && refreshAttempt < 19) {
        console.log(`Refresh attempt ${refreshAttempt + 1} rate limited, will retry`);
      } else {
        console.error("Failed to refresh cloud control resource");
        console.log(child.stdout);
        console.error(`Refresh attempt ${refreshAttempt + 1} failed:`, child.stderr);
      }

      if (isRateLimited && refreshAttempt < 19) {
        refreshAttempt++;
        const exponentialDelay = Math.min(baseDelay * Math.pow(2, refreshAttempt - 1), maxDelay);
        const jitter = Math.random() * 0.3 * exponentialDelay;
        const finalDelay = exponentialDelay + jitter;

        console.log(`[VPN-REFRESH] Rate limited on attempt ${refreshAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
        await delay(finalDelay);
        continue;
      } else {
        return {
          payload: resource,
          status: "error",
          message:
            `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
        };
      }
    } else {
      console.log(`[VPN-REFRESH] Refresh successful on attempt ${refreshAttempt + 1}`);
      resourceResponse = JSON.parse(child.stdout);
      break;
    }
  }

  const payload = JSON.parse(
    resourceResponse["ResourceDescription"]["Properties"],
  );

  const transitGatewayId = _.get(
    component,
    "properties.domain.TransitGatewayId",
  );

  if (transitGatewayId) {
    try {
      const vpnConnectionId = payload.VpnConnectionId || name;
      let attachmentAttempt = 0;
      let attachmentSuccess = false;

      console.log(`[VPN-REFRESH] Starting Transit Gateway attachment lookup for VPN: ${vpnConnectionId}`);

      // Retry attachment lookup if rate limited
      while (attachmentAttempt < 20 && !attachmentSuccess) {
        const attachmentChild = await siExec.waitUntilEnd("aws", [
          "ec2",
          "describe-transit-gateway-attachments",
          "--region",
          _.get(component, "properties.domain.extra.Region", ""),
          "--filters",
          `Name=resource-type,Values=vpn`,
          `Name=transit-gateway-id,Values=${transitGatewayId}`,
          `Name=resource-id,Values=${vpnConnectionId}`,
          "--query",
          "TransitGatewayAttachments[0]",
        ]);

        console.log(`Attachment lookup attempt ${attachmentAttempt + 1}: AWS CLI exit code: ${attachmentChild.exitCode}`);

        if (attachmentChild.exitCode !== 0) {
          const isRateLimited = attachmentChild.stderr.includes("Throttling") ||
                               attachmentChild.stderr.includes("TooManyRequests") ||
                               attachmentChild.stderr.includes("RequestLimitExceeded") ||
                               attachmentChild.stderr.includes("ThrottlingException");

          if (isRateLimited && attachmentAttempt < 19) {
            console.log(`Attachment lookup attempt ${attachmentAttempt + 1} rate limited, will retry`);
            attachmentAttempt++;
            const exponentialDelay = Math.min(baseDelay * Math.pow(2, attachmentAttempt - 1), maxDelay);
            const jitter = Math.random() * 0.3 * exponentialDelay;
            const finalDelay = exponentialDelay + jitter;

            console.log(`[VPN-REFRESH] Attachment lookup rate limited on attempt ${attachmentAttempt}, waiting ${Math.round(finalDelay)}ms before retry`);
            await delay(finalDelay);
            continue;
          } else {
            console.error(`Attachment lookup attempt ${attachmentAttempt + 1} failed:`, attachmentChild.stderr);
            break;
          }
        } else {
          console.log(`[VPN-REFRESH] Attachment lookup successful on attempt ${attachmentAttempt + 1}`);
          attachmentSuccess = true;

          if (
            attachmentChild.stdout &&
            attachmentChild.stdout.trim() !== "null"
          ) {
            const attachment = JSON.parse(attachmentChild.stdout);
            if (attachment?.TransitGatewayAttachmentId) {
              console.log(
                `Found Transit Gateway Attachment ID: ${attachment.TransitGatewayAttachmentId}`,
              );
              payload.TransitGatewayAttachmentId =
                attachment.TransitGatewayAttachmentId;
            }
          }
        }
      }
    } catch (error) {
      console.error("Failed to get attachment info:", error);
    }
  }

  console.log(`[VPN-REFRESH] Final result: success, returning VPN payload`);
  return {
    payload,
    status: "ok",
  };
}
