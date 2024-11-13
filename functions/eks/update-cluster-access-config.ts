async function main(component: Input) {
    const resource = component.properties.resource?.payload;
    if (!resource) {
        return {
            status: component.properties.resource?.status ?? "ok",
            message: component.properties.resource?.message,
        };
    }

    let json = {
        "accessConfig": {
            "authenticationMode": component.properties.domain.accessConfig.authenticationMode,
        },
        "name": resource.name,
    };

    const updateResp = await siExec.waitUntilEnd("aws", [
        "eks",
        "update-cluster-config",
        "--cli-input-json",
        JSON.stringify(json),
        "--region",
        component.properties.domain?.extra.Region || "",
    ]);

    if (updateResp.exitCode !== 0) {
        console.error(updateResp.stderr);
        return {
            status: "error",
            payload: resource,
            message: `Unable to update the EKS Cluster Access Config, AWS CLI 2 exited with non zero code: ${updateResp.exitCode}`,
        };
    }

    return {
        payload: resource,
        status: "ok"
    };
}