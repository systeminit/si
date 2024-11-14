async function main(component: Input): Promise<Output> {
    // Get the name from the resourceId (or from payload, for backwards compatibility if this isn't a purely new asset).
    const payload = component.properties?.resource?.payload;
    const name = component.properties?.si?.resourceId ?? payload?.name;
    if (!name) {
        return {
            status: "error",
            message: "No resourceId present",
        };
    }

    // Run the AWS CLI command.
    const cliInput = { name };
    const Region = component.properties?.domain?.extra?.Region ?? "";
    const child = await siExec.waitUntilEnd("aws", [
        "eks",
        "delete-cluster",
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

    // The resource is successfully deleted; remove the payload.
    return {
        status: "ok",
        payload: null,
    };
}