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
    console.error("Failed to refresh cloud control resource");
    console.log(child.stdout);
    console.error(child.stderr);
    return {
      payload: resource,
      status: "error",
      message:
        `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
    };
  }

  const resourceResponse = JSON.parse(child.stdout);
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

      if (
        attachmentChild.exitCode === 0 &&
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
    } catch (error) {
      console.error("Failed to get attachment info:", error);
    }
  }

  return {
    payload,
    status: "ok",
  };
}
