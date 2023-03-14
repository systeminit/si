async function deleteResource(component) {
    const resource = component.properties.resource?.value;
    // Now, delete the Ec2 Instance.
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "terminate-instances",
        "--region",
        component.properties.domain.region,
        "--instance-ids",
        resource.InstanceId,
    ]);

    if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
            status: "error",
            value: resource,
            message: `Unable to delete Ec2 Instance, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
    }

    return {value: null, status: "ok"};
}
