async function refresh(component: Input): Promise<Output> {
  // Refresh the key pair.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-images",
    "--region",
    component.properties.domain.region,
    "--cli-input-json",
    component.properties.code["si:generateAwsAmiJSON"]?.code,
  ]);

  if (child.stderr.includes("InvalidAMIID.NotFound")) {
    console.log(`AMI Id: ${component.properties.domain.ImageId}`);
    console.error(child.stderr);
    return {
      status: "error",
      message: `AMI ${component.properties.domain.ImageId} not found in region ${component.properties.domain.region} (InvalidAMIID.NotFound)`,
    }
  }
  
  if (child.stderr.includes("InvalidAMIID.Malformed")) {
    console.log(`AMI Id: ${component.properties.domain.ImageId}`);
    console.error(child.stderr);
    return {
      status: "error",
      value: component.properties.resource?.value,
      message: "AMI ${component.properties.domain.ImageId} is invalid (InvalidAMIID.Malformed)",
    }
  }

  if (child.exitCode !== 0) {
    console.log(`AMI Id: ${component.properties.domain.ImageId}`);
    console.error(child.stderr);
    return {
      status: "error",
      value: component.properties.resource?.value,
      message: `AWS CLI 2 "aws ec2 describe-images" returned non zero exit code (${child.exitCode})`,
    }
  }

  const images = (JSON.parse(child.stdout) || {})["Images"];
  if (!images || images.length === 0) {
    console.log(`AMI Id: ${component.properties.domain.ImageId}`);
    console.log(child.stdout);
    return {
      status: "error",
      value: component.properties.resource?.value,
      message: `AMI ${component.properties.domain.ImageId} not found in region ${component.properties.domain.region}`,
    }
  }

  return { value: images[0], status: "ok" };
}
