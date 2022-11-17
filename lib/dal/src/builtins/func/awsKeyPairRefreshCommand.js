async function refresh(component) {
  const resource = component.properties.resource;
  if (!resource || resource === "null") return null;

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-key-pairs",
    "--key-pair-ids",
    resource.KeyPairId,
  ]);

  if (child.exitCode !== 0) {
    throw new Error(`Failure running aws ec2 describe-key-pairs (${child.exitCode}): ${child.stderr}`);
  }

  console.log(child.stdout);
  return { value: JSON.parse(child.stdout).KeyPairs[0] };
}
