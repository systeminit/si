async function main({
  thisComponent,
}: Input): Promise<Output> {
  // Changed: access via domain.properties
  const properties = thisComponent.properties.domain.properties;
  const region = _.get(thisComponent, [
    "properties",
    "domain",
    "extra",
    "Region",
  ]);
  const resourceId = _.get(thisComponent, ["properties", "si", "resourceId"]);

  console.log("=== ROUTE53 IMPORT WITH 'ops' KEY ===");
  console.log(
    "Starting Route53 record import for:",
    JSON.stringify(properties, null, 2),
  );
  console.log("Resource ID:", resourceId);
  console.log("Region:", region);

  if (!resourceId) {
    throw new Error(
      "Resource ID is required for import. Format should be like: tonys-chips.local.:SOA",
    );
  }

  if (!region) {
    throw new Error("Region is required for AWS CLI operations");
  }

  // Parse resource ID to get hosted zone ID, name and type
  const parts = resourceId.split(":");
  if (parts.length !== 2) {
    throw new Error(
      "Invalid resource ID format. Expected format: recordname:type (e.g., tonys-chips.local.:SOA)",
    );
  }

  const recordName = parts[0];
  const recordType = parts[1];

  console.log("Parsed - Record Name:", recordName, "Type:", recordType);

  // First, find the hosted zone for this domain
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

  // Find the hosted zone that matches this record
  let hostedZoneId = null;
  for (const zone of zonesResponse.HostedZones || []) {
    console.log("Checking zone:", zone.Name, "against record:", recordName);
    if (recordName === zone.Name) {
      hostedZoneId = zone.Id.replace("/hostedzone/", "");
      console.log("Found matching hosted zone:", hostedZoneId);
      break;
    }
  }

  if (!hostedZoneId) {
    throw new Error(`Could not find hosted zone for record: ${recordName}`);
  }

  // Get the specific record
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
  console.log(
    "Total records found:",
    recordsResponse.ResourceRecordSets?.length || 0,
  );

  // Find the specific record we want to import
  const targetRecord = recordsResponse.ResourceRecordSets?.find((record) => {
    const nameMatch = record.Name === recordName;
    const typeMatch = record.Type === recordType;
    console.log(
      `Checking record: ${record.Name} (${record.Type}) - Name match: ${nameMatch}, Type match: ${typeMatch}`,
    );
    return nameMatch && typeMatch;
  });

  if (!targetRecord) {
    console.log("Available records:");
    recordsResponse.ResourceRecordSets?.forEach((record) => {
      console.log(`  - ${record.Name} (${record.Type})`);
    });
    throw new Error(`Record not found: ${recordName} (${recordType})`);
  }

  console.log("Found target record:", JSON.stringify(targetRecord, null, 2));

  // Changed: nest domainProps under properties
  const domainProps = {
    properties: {
      Name: targetRecord.Name,
      Type: targetRecord.Type,
      HostedZoneId: hostedZoneId,
      TTL: targetRecord.TTL?.toString(),
      ResourceRecords:
        targetRecord.ResourceRecords?.map((rr) => rr.Value) || [],
    },
  };

  console.log("Built domain properties:", JSON.stringify(domainProps, null, 2));

  // Build the ops structure (not operations!)
  const opsStructure = {
    actions: {
      self: {
        add: [],
        remove: ["create", "refresh"],
      },
    },
    update: {
      self: {
        properties: {
          domain: domainProps,
          resource: targetRecord,
        },
      },
    },
  };

  console.log("Built ops structure:", JSON.stringify(opsStructure, null, 2));

  // Build the final return object with 'ops' not 'operations'
  const returnObject = {
    status: "ok",
    message: "Imported Resource",
    ops: opsStructure,
  };

  console.log(
    "Final return object with 'ops':",
    JSON.stringify(returnObject, null, 2),
  );
  console.log("=== ROUTE53 IMPORT END ===");

  return returnObject;
}
