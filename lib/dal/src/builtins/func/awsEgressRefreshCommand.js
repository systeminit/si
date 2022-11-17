async function refresh(component) {
  const resource = component.properties.resource;
  if (!resource || resource === "null") return null;

  let groupId;
  for (const rule of resource.SecurityGroupRules) {
    if (groupId !== undefined && groupId !== rule.GroupId) {
      throw new Error(`Egress referenced multiple group ids: ${JSON.stringify(resource.SecurityGroupRules)}`);
    }
    groupId = rule.GroupId;
  }

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-security-groups",
    "--group-ids",
    groupId,
  ]);

  if (child.exitCode !== 0) {
    throw new Error(`Failure running aws ec2 describe-security-groups (${child.exitCode}): ${child.stderr}`);
  }

  console.log(child.stdout);
  const securityGroup = JSON.parse(child.stdout).SecurityGroups[0];
  for (const rule of resource.SecurityGroupRules) {
    for (const IpPermission of securityGroup.IpPermissionsEgress) {
      if (IpPermission.ToPort === rule.ToPort &&
        IpPermission.FromPort === rule.FromPort &&
        IpPermission.IpProtocol === rule.IpProtocol) {

        for (const range in IpPermission.IpRanges) {
          if (range.CidrIp === rule.CidrIpv4) {
            // Should we update some metadata if available?
            return resource;
          }
        }
      }
    }
  }

  // Egress wasn't found, resetting resource
  return null;
}
