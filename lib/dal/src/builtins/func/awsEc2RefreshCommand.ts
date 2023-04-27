async function refresh(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message
    };
  }

  const instances = Array.isArray(resource) ? resource : [resource];
  const instanceIds = instances.flatMap((i) => i.Instances).map((i) => i.InstanceId).filter((id) => !!id);
  if (!instanceIds || instanceIds.length === 0) return {
    status: "error",
    payload: resource,
    message: "No EC2 instance id found"
  };

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-instances",
    "--instance-ids",
    ...instanceIds,
    "--region",
    component.properties.domain.region,
  ]);

  if (child.stderr.includes("InvalidInstance.NotFound")) {
    console.log(`Instance Ids: ${instanceIds}`);
    console.error(child.stderr);
    return {
      status: "error",
      message: `EC2 Instance not found (InvalidInstance.NotFound)`,
    }
  }

  if (child.stderr.includes("InvalidInstanceID.Malformed")) {
    console.log(`Instance Ids: ${instanceIds}`);
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: "EC2 Instance Id is invalid (InvalidInstanceID.Malformed)",
    }
  }

  if (child.exitCode !== 0) {
    console.log(`Instance Ids: ${instanceIds}`);
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: `AWS CLI 2 "aws ec2 describe-instances" returned non zero exit code (${child.exitCode})`,
    }
  }

  const object = JSON.parse(child.stdout);

  if (!object.Reservations || object.Reservations.length === 0) {
    console.log(`Instance Ids: ${instanceIds}`);
    console.error(child.stdout);
    return {
      status: "error",
      message: "Instance not found in payload returned by AWS",
    }
  }

  let status: "ok" | "warning" | "error" = "ok";
  let message;
  for (const reservation of object.Reservations) {
    for (const instance of reservation.Instances) {
      if (["terminated", "shutting-down", "stopped", "stopping"].includes(instance.State.Name)) {
        status = "error";
        message = `Instance not running, state: ${instance.State.Name}`;
        break;
      } else if (instance.State.Name === "pending") {
        status = "warning";
        message = `Instance not running, state: ${instance.State.Name}`;
        break;
      }
    }
  }

  return { payload: object.Reservations, status, message };
}
