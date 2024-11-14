async function main(component: Input) {
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
    const cliInput = {
        name,
        accessConfig: {
            authenticationMode: component.properties?.domain?.accessConfig.authenticationMode,
        },
    };
    const Region = component.properties?.domain?.extra?.Region ?? "";
    const child = await siExec.waitUntilEnd("aws", [
        "eks",
        "update-cluster-config",
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
        }
    }

    return {
        status: "ok",
        // TODO probably don't update the payload unless it's full
        payload
    };
}