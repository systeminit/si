async function main(component: Input): Promise<Output> {
    let name = component.properties?.si?.resourceId;
    const resource = component.properties.resource?.payload;
    if (!name) {
        name = resource.name;
    }
    if (!name) {
        return {
            status: component.properties.resource?.status ?? "error",
            message: "Could not refresh, no resourceId present for EKS Cluster component",
        };
    }

    const cliArguments = {};
    _.set(
        cliArguments,
        "name",
        name,
    );

    const child = await siExec.waitUntilEnd("aws", [
        "eks",
        "describe-cluster",
        "--region",
        _.get(component, "properties.domain.extra.Region", ""),
        "--cli-input-json",
        JSON.stringify(cliArguments),
    ]);

    if (child.exitCode !== 0) {
        console.log(`cluster Name: ${name}`);
        console.error(child.stderr);
        if (child.stderr.includes("ResourceNotFoundException")) {
            console.log("EKS Cluster not found  upstream (ResourceNotFoundException) so removing the resource")
            return {
                status: "ok",
                payload: null,
            };
        }
        return {
            status: "error",
            payload: resource,
            message: `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
        };
    }

    const object = JSON.parse(child.stdout).cluster;
    return {
        payload: object,
        status: "ok",
    };
}