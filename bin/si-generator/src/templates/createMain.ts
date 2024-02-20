export const partial = `
async function main(component: Input): Promise < Output > {
    if (component.properties.resource?.payload) {
        return {
            status: "error",
            message: "Resource already exists",
            payload: component.properties.resource.payload,
        };
    }

    const code = component.properties.code?.["si:genericAwsCreate"]?.code;
    const domain = component.properties?.domain;

    const child = await siExec.waitUntilEnd("aws", [
        "<%= it.options.awsService %>",
        "<%= it.options.awsCommand %>",
        "--region",
        domain?.extra?.Region || "",
        "--cli-input-json",
        code || "",
    ]);

    if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
            status: "error",
            message: \`Unable to create; AWS CLI 2 exited with non zero code: \${child.exitCode}\`,
        };
    }

    const response = JSON.parse(child.stdout);

    return {
        payload: response,
        status: "ok"
    };
}`
