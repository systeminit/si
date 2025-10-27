async function main(component: Input): Promise<Output> {
  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  const domain = component.properties?.domain;
  const hostedZoneId = domain?.HostedZoneId;
  const name = domain?.Name;
  const type = domain?.Type;

  if (!hostedZoneId) {
    return {
      status: "error",
      message: "HostedZoneId is required",
    };
  }

  if (!name) {
    return {
      status: "error",
      message: "Name is required",
    };
  }

  if (!type) {
    return {
      status: "error",
      message: "Type is required",
    };
  }

  const changeBatch = {
    Changes: [
      {
        Action: "CREATE",
        ResourceRecordSet: {
          Name: name,
          Type: type,
          TTL: domain?.TTL ? parseInt(domain.TTL) : undefined,
          ResourceRecords: domain?.ResourceRecords?.map((record: string) => ({
            Value: record,
          })),
          AliasTarget: domain?.AliasTarget
            ? {
              DNSName: domain.AliasTarget.DNSName,
              EvaluateTargetHealth: domain.AliasTarget.EvaluateTargetHealth,
              HostedZoneId: domain.AliasTarget.HostedZoneId,
            }
            : undefined,
          SetIdentifier: domain?.SetIdentifier,
          Weight: domain?.Weight,
          Region: domain?.Region,
          GeoLocation: domain?.GeoLocation,
          Failover: domain?.Failover,
          MultiValueAnswer: domain?.MultiValueAnswer,
          HealthCheckId: domain?.HealthCheckId,
        },
      },
    ],
  };

  const child = await siExec.waitUntilEnd("aws", [
    "route53",
    "change-resource-record-sets",
    "--hosted-zone-id",
    hostedZoneId,
    "--change-batch",
    JSON.stringify(changeBatch),
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message:
        `Unable to create; AWS CLI exited with non zero code: ${child.exitCode}`,
    };
  }

  const response = JSON.parse(child.stdout);
  console.log("Change Info:", response.ChangeInfo);

  return {
    resourceId: `${hostedZoneId}:${name}:${type}`,
    status: "ok",
  };
}
