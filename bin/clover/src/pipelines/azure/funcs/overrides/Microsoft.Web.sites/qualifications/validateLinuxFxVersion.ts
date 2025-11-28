async function main(component: Input): Promise < Output > {

    const linuxFxVersion = component.domain.properties?.siteConfig?.linuxFxVersion;
    if (!linuxFxVersion) {
        return {
            result: 'success',
            message: 'No linuxFxVersion to qualify'
        };
    }

    const args = ["functionapp", "list-runtimes", "--os", "linux"];
    const runtimesResponse = await siExec.waitUntilEnd("az", args);
    if (runtimesResponse.exitCode !== 0) {
        console.log(runtimesResponse.stdout);
        console.error(runtimesResponse.stderr);
        return {
            result: "failure",
            message: `Unable to list-runtimes for functionapp from AZ CLI: ${runtimesResponse.exitCode}`,
        };
    }
    let runtimes = JSON.parse(runtimesResponse.stdout);
    const found = runtimes.some(
        r => r.linux_fx_version === linuxFxVersion
    );

    if (!found) {
        return {
            result: "failure",
            message: `linuxFxVersion ${linuxFxVersion} is not a valid runtime version`
        };
    }

    return {
        result: "success",
        message: "linuxFxVersion ${linuxFxVersion} is valid!"
    };
}
