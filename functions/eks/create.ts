async function main(component: Input): Promise<Output> {
    if (component.properties.resource?.payload) {
        return {
            status: "error",
            message: "Resource already exists",
            payload: component.properties.resource.payload,
        };
    }

    const code = component.properties.code?.["awsEksClusterCodeGen"]?.code;
    const domain = component.properties?.domain;

    const child = await siExec.waitUntilEnd("aws", [
        "eks",
        "create-cluster",
        "--region",
        domain?.extra?.Region || "",
        "--cli-input-json",
        code || "",
    ]);

    if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
            status: "error",
            message: `Unable to create; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
    }

    const response = JSON.parse(child.stdout).cluster;

    return {
        resourceId: response.name,
        status: "ok",
    };
}