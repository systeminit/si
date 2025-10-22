async function main({
  thisComponent,
}: Input): Promise<Output> {
  const domain = thisComponent.properties.domain;
  const region = _.get(thisComponent, [
    "properties",
    "domain",
    "extra",
    "Region",
  ]);

  console.log("=== ROUTE53 DISCOVER RECORD SETS ===");
  console.log("Starting Route53 record discovery");
  console.log("Domain attributes:", JSON.stringify(domain, null, 2));
  console.log("Region:", region);

  if (!region) {
    throw new Error("Region is required for AWS CLI operations");
  }

  // Get hosted zone ID - either directly or by looking up the name
  let hostedZoneId = domain.HostedZoneId;
  const hostedZoneName = domain.HostedZoneName;

  // If we don't have a hosted zone ID but have a name, look it up
  if (!hostedZoneId && hostedZoneName) {
    console.log("Looking up hosted zone by name:", hostedZoneName);
    const listZonesChild = await siExec.waitUntilEnd("aws", [
      "route53",
      "list-hosted-zones",
      "--region",
      region,
    ]);

    if (listZonesChild.exitCode !== 0) {
      console.error("Failed to list hosted zones:", listZonesChild.stderr);
      throw new Error(`Failed to list hosted zones: ${listZonesChild.stderr}`);
    }

    const zonesResponse = JSON.parse(listZonesChild.stdout);
    console.log("Found hosted zones:", zonesResponse.HostedZones?.length || 0);

    // Find the hosted zone that matches the name
    for (const zone of zonesResponse.HostedZones || []) {
      if (zone.Name === hostedZoneName) {
        hostedZoneId = zone.Id.replace("/hostedzone/", "");
        console.log("Found hosted zone ID:", hostedZoneId);
        break;
      }
    }

    if (!hostedZoneId) {
      throw new Error(
        `Could not find hosted zone with name: ${hostedZoneName}`,
      );
    }
  }

  if (!hostedZoneId) {
    throw new Error(
      "Either HostedZoneId or HostedZoneName must be provided for discovery",
    );
  }

  console.log("Discovering record sets in hosted zone:", hostedZoneId);

  // List all record sets in the hosted zone
  const listRecordsChild = await siExec.waitUntilEnd("aws", [
    "route53",
    "list-resource-record-sets",
    "--hosted-zone-id",
    hostedZoneId,
    "--region",
    region,
  ]);

  if (listRecordsChild.exitCode !== 0) {
    console.error("Failed to list records:", listRecordsChild.stderr);
    throw new Error(`Failed to list records: ${listRecordsChild.stderr}`);
  }

  const recordsResponse = JSON.parse(listRecordsChild.stdout);
  const recordSets = recordsResponse.ResourceRecordSets || [];
  console.log("Total record sets found:", recordSets.length);

  // Build component specs for each record set
  const specs: Output["ops"]["create"] = {};

  for (const record of recordSets) {
    console.log(`Processing record: ${record.Name} (${record.Type})`);

    // Create a unique component name
    // Remove trailing dot from name and replace dots with dashes
    const baseName = record.Name.replace(/\.$/, "").replace(/\./g, "-");
    const componentName = `${baseName}-${record.Type}`;

    // Build the domain properties
    const domainProps: any = {
      Name: record.Name,
      Type: record.Type,
      HostedZoneId: hostedZoneId,
    };

    // Add optional properties if they exist
    if (record.TTL !== undefined) {
      domainProps.TTL = record.TTL.toString();
    }

    if (record.ResourceRecords && record.ResourceRecords.length > 0) {
      domainProps.ResourceRecords = record.ResourceRecords.map((rr) =>
        rr.Value
      );
    }

    if (record.AliasTarget) {
      domainProps.AliasTarget = {
        DNSName: record.AliasTarget.DNSName,
        HostedZoneId: record.AliasTarget.HostedZoneId,
        EvaluateTargetHealth: record.AliasTarget.EvaluateTargetHealth,
      };
    }

    if (record.SetIdentifier) {
      domainProps.SetIdentifier = record.SetIdentifier;
    }

    if (record.Weight !== undefined) {
      domainProps.Weight = record.Weight;
    }

    if (record.Region) {
      domainProps.Region = record.Region;
    }

    if (record.GeoLocation) {
      domainProps.GeoLocation = record.GeoLocation;
    }

    if (record.Failover) {
      domainProps.Failover = record.Failover;
    }

    if (record.MultiValueAnswer !== undefined) {
      domainProps.MultiValueAnswer = record.MultiValueAnswer;
    }

    if (record.HealthCheckId) {
      domainProps.HealthCheckId = record.HealthCheckId;
    }

    if (record.CidrRoutingConfig) {
      domainProps.CidrRoutingConfig = record.CidrRoutingConfig;
    }

    if (record.GeoProximityLocation) {
      domainProps.GeoProximityLocation = record.GeoProximityLocation;
    }

    // Build the component spec
    specs[componentName] = {
      kind: "AWS::Route53::RecordSet",
      properties: {
        si: {
          name: componentName,
        },
        domain: domainProps,
        resource: record,
      },
      attributes: {
        "/domain/extra/Region": {
          $source: thisComponent.sources["/domain/extra/Region"],
        },
        "/secrets/AWS Credential": {
          $source: thisComponent.sources["/secrets/AWS Credential"],
        },
      },
    };

    console.log(`Created spec for: ${componentName}`);
  }

  console.log(`Created ${Object.keys(specs).length} component specs`);
  console.log("=== ROUTE53 DISCOVER END ===");

  return {
    status: "ok",
    ops: {
      create: specs,
    },
  };
}
