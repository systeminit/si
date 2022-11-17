async function create(component) {
  if (component.properties.resource !== undefined
    && component.properties.resource !== null
    && component.properties.resource !== ""
    && component.properties.resource !== "null") {
    throw new Error("resource already exists");
  }

  // Now, create the ec2
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "run-instances",
    "--region",
    component.properties.domain.region,
    "--cli-input-json",
    component.properties.code["si:generateAwsEc2JSON"]?.code,
  ]);

  if (child.exitCode !== 0) {
    throw new Error(`Failure running aws ec2 run-instances (${child.exitCode}): ${child.stderr}`);
  }

  console.log(child.stdout);
  return { value: JSON.parse(child.stdout) };
}
