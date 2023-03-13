async function deleteResource(component) {
    const resource = component.properties.resource?.value[0];

    // Now, delete the ingress
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "revoke-security-group-ingress",
        "--region",
        component.properties.domain.region,
        "--security-group-rule-ids",
        resource.SecurityGroupRuleId,
        "--group-id",
        component.properties.domain.GroupId,
    ]);

    if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
            status: "error",
            value: resource,
            message: `Unable to delete Ingress, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        }
    }

    return {value: null, status: "ok"};
}
