async function main(component: Input): Promise<Output> {
    // If the resource already exists, return an error.
    const payload = component.properties?.resource.payload ?? {};
    if (component.properties?.si?.resourceId ?? payload.length) {
        return {
            status: "error",
            message: "Resource already exists",
        };
    }

    // Run the AWS CLI command.
    const cliInput = component.properties?.code?.["awsEksClusterCodeGen"]?.code ?? {};
    const Region = component.properties?.domain?.extra?.Region ?? "";
    const child = await siExec.waitUntilEnd("aws", [
        "eks",
        "create-cluster",
        "--cli-input-json",
        JSON.stringify(cliInput),
        "--region",
        Region,
    ], { stderr: ["inherit", "pipe"] });

    // Return an error if the CLI command failed. (Handle specific error cases here.)
    if (child.failed) {
        return {
            status: "error",
            message: child.message,
        };
    }

    // The resource is successfully deleted; ship the payload
    const response = JSON.parse(child.stdout);
    return {
        status: "ok",
        resourceId: response.cluster.name,
    };
}