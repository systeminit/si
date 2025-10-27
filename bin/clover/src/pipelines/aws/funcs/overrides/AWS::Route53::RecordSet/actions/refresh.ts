async function main(component: Input): Promise<Output> {
  const domain = component.properties?.domain;
  const hostedZoneId = domain?.HostedZoneId;
  const name = domain?.Name;
  const type = domain?.Type;

  if (!hostedZoneId) {
    return {
      status: "error",
      message: "HostedZoneId is required for refresh",
    };
  }

  if (!name) {
    return {
      status: "error",
      message: "Name is required for refresh",
    };
  }

  if (!type) {
    return {
      status: "error",
      message: "Type is required for refresh",
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "route53",
    "list-resource-record-sets",
    "--hosted-zone-id",
    hostedZoneId,
    "--query",
    `ResourceRecordSets[?Name=='${name}' && Type=='${type}']`,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message: `Refresh error; exit code ${child.exitCode}`,
    };
  }

  const recordSets = JSON.parse(child.stdout);

  if (!recordSets || recordSets.length === 0) {
    console.log("Record set not found upstream, removing resource");
    return {
      status: "ok",
      payload: null,
    };
  }

  const recordSet = recordSets[0];

  const payload = {
    Name: recordSet.Name,
    Type: recordSet.Type,
    TTL: recordSet.TTL,
    ResourceRecords: recordSet.ResourceRecords?.map((rr: any) => rr.Value),
    AliasTarget: recordSet.AliasTarget,
    SetIdentifier: recordSet.SetIdentifier,
    Weight: recordSet.Weight,
    Region: recordSet.Region,
    GeoLocation: recordSet.GeoLocation,
    Failover: recordSet.Failover,
    MultiValueAnswer: recordSet.MultiValueAnswer,
    HealthCheckId: recordSet.HealthCheckId,
    HostedZoneId: hostedZoneId,
  };

  return {
    payload,
    status: "ok",
  };
}
