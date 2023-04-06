async function create(component) {
  if (component.properties.resource?.value) {
    return {
      status: "error",
      message: "Resource already exists",
      value: component.properties.resource.value,
    }
  }

  // Now, create the ingress
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "authorize-security-group-ingress",
    "--region",
    component.properties.domain.region,
    "--cli-input-json",
    component.properties.code["si:generateAwsIngressJSON"]?.code,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message: `Unable to create Ingress, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    }
  }

  return { value: JSON.parse(child.stdout).SecurityGroupRules, status: "ok" };
}
