async function deleteResource(component: Input): Promise<Output> {
    const resource = component.properties.resource?.value[0];

    if (resource.SecurityGroupRuleId === undefined) {
        console.error("unable to find a valid SecurityGroupRuleID");
        return {
            status: "error",
            value: resource,
            message: `Unable to delete Egress Rule, unable to find a valid SecurityGroupRuleId`,
        }
    }

    if (resource.GroupId === undefined) {
        console.error("unable to find a valid GroupID");
        return {
            status: "error",
            value: resource,
            message: `Unable to delete Egress Rule, unable to find a valid GroupID`,
        }
    }

    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "revoke-security-group-egress",
        "--region",
        component.properties.domain.region,
        "--security-group-rule-ids",
        resource.SecurityGroupRuleId,
        "--group-id",
        resource.GroupId,
    ]);

    if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
            status: "error",
            value: resource,
            message: `Unable to delete Egress Rule, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        }
    }

    return {value: null, status: "ok"};
}
