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
        "describe-cluster",
        "--cli-input-json",
        JSON.stringify(cliInput),
        "--region",
        Region,
    ], { stderr: ["inherit", "pipe"] });

    // Return an error if the CLI command failed. (Handle specific error cases here.)
    if (child.failed) {
        // Remove the payload if the resource no longer exists in AWS
        const NOT_FOUND_MESSAGE = "ResourceNotFoundException"
        if (child.stderr?.includes(NOT_FOUND_MESSAGE)) {
            console.log(`Resource not found upstream (${NOT_FOUND_MESSAGE}) so removing the resource.`)
            return {
                status: "ok",
                payload: null
            };
        }
        return {
            status: "error",
            message: child.message
        }
    }

    // Return the updated resource.
    const response = JSON.parse(child.stdout);
    return {
        status: "ok",
        payload: response.cluster
    };
}
