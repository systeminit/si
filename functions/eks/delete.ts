async function main(component: Input): Promise<Output> {
    const cliArguments = {};
    _.set(
        cliArguments,
        "name",
        _.get(component, "properties.resource.payload.name"),
    );

    const child = await siExec.waitUntilEnd("aws", [
        "eks",
        "delete-cluster",
        "--region",
        _.get(component, "properties.domain.extra.Region", ""),
        "--cli-input-json",
        JSON.stringify(cliArguments),
    ]);

    if (child.exitCode !== 0) {
        const payload = _.get(component, "properties.resource.payload");
        if (payload) {
            return {
                status: "error",
                payload,
                message: `Delete error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
            };
        } else {
            return {
                status: "error",
                message: `Delete error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
            };
        }
    }

    return {
        payload: null,
        status: "ok",
    };
}