async function main(component: Input): Promise<Output> {
  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  // Changed: access via domain.properties
  const properties = component.properties?.domain?.properties;
  const hostedZoneId = properties?.HostedZoneId;
  const name = properties?.Name;
  const type = properties?.Type;

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
          TTL: properties?.TTL ? parseInt(properties.TTL) : undefined,
          ResourceRecords: properties?.ResourceRecords?.map(
            (record: string) => ({
              Value: record,
            }),
          ),
          AliasTarget: properties?.AliasTarget
            ? {
                DNSName: properties.AliasTarget.DNSName,
                EvaluateTargetHealth: properties.AliasTarget.EvaluateTargetHealth,
                HostedZoneId: properties.AliasTarget.HostedZoneId,
              }
            : undefined,
          SetIdentifier: properties?.SetIdentifier,
          Weight: properties?.Weight,
          Region: properties?.Region,
          GeoLocation: properties?.GeoLocation,
          Failover: properties?.Failover,
          MultiValueAnswer: properties?.MultiValueAnswer,
          HealthCheckId: properties?.HealthCheckId,
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
      message: `Unable to create; AWS CLI exited with non zero code: ${child.exitCode}`,
    };
  }

  const response = JSON.parse(child.stdout);
  console.log("Change Info:", response.ChangeInfo);

  return {
    resourceId: `${hostedZoneId}:${name}:${type}`,
    status: "ok",
  };
}
