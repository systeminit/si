async function refresh(component) {
  const resource = component.properties.resource?.value;
  if (!resource) return { status: "ok" };

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-key-pairs",
    "--key-pair-ids",
    resource.KeyPairId,
  ]);

  if (child.stderr.includes("InvalidKeyPair.NotFound")) {
    console.log(`Key Pair Id: ${resource.KeyPairId}`);
    console.error(child.stderr);
    return {
      status: "error",
      message: `Key Pair not found (InvalidKeyPair.NotFound)`,
    }
  }
  
  if (child.stderr.includes("InvalidParameterValue")) {
    console.log(`Key Pair Id: ${resource.KeyPairId}`);
    console.error(child.stderr);
    return {
      status: "error",
      value: resource,
      message: "Key Pair Id is invalid (InvalidParameterValue)",
    }
  }

  if (child.exitCode !== 0) {
    console.log(`Key Pair Id: ${resource.KeyPairId}`);
    console.error(child.stderr);
    return {
      value: resource,
      status: "error",
      message: `AWS CLI 2 "aws ec2 describe-key-pairs" returned non zero exit code (${child.exitCode})`,
    }
  }

  const object = JSON.parse(child.stdout);

  if (!object.KeyPairs || object.KeyPairs.length === 0) {
    console.log(`Key Pair Id: ${resource.KeyPairId}`);
    console.error(child.stdout);
    return {
      status: "error",
      value: resource,
      message: "Key Pair not found in payload returned by AWS, but it should be there",
    }
  }

  return { value: object.KeyPairs[0], status: "ok" };
}
