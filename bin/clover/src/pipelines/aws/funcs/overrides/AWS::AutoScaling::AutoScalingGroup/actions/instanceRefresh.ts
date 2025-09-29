async function main(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message,
    };
  }

  if (!resource.AutoScalingGroupName) {
    return {
      status: "error",
      payload: resource,
      message: "No AutoScaling Group name found",
    };
  }

  const minHealthyPercentage =
    component.properties.domain?.InstanceMaintenancePolicy
      ?.MinHealthyPercentage;

  const defaultInstanceWarmup =
    component.properties.domain?.DefaultInstanceWarmup;

  if (!minHealthyPercentage || !defaultInstanceWarmup) {
    return {
      status: "error",
      payload: resource,
      message:
        "Recycling an AutoScaling Group instances requires a MinHealthyPercentage and DefaultInstanceWarmup. Please ensure you set these before re-trying the action",
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "autoscaling",
    "start-instance-refresh",
    "--auto-scaling-group-name",
    resource.AutoScalingGroupName,
    "--strategy",
    "Rolling",
    "--preferences",
    `{"MinHealthyPercentage":${minHealthyPercentage},"InstanceWarmup":${defaultInstanceWarmup}}`,
    "--region",
    component.properties.domain?.extra.Region,
  ]);

  if (child.exitCode !== 0) {
    console.log(`AutoScaling group: ${resource.AutoScalingGroupName}`);
    console.error(child.stderr);
    return {
      payload: resource,
      status: "error",
      message: `AWS CLI 2 "aws autoscaling refresh-instances" returned non zero exit code (${child.exitCode})`,
    };
  }

  return {
    payload: resource,
    status: "ok",
  };
}
