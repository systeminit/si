async function create(component) {
    console.log(component);

    if (component.resource) {
        throw new Error("resource already exists");
    }

    const child = await siExec.waitUntilEnd("skopeo", ["inspect", "--override-os", "linux", "--override-arch", "amd64", `docker://docker.io/${component.properties.domain.image}`]);
    if (child.exitCode !== 0) {
        throw new Error(child.stderr);
    }

    console.log(child.stdout);
    const object = JSON.parse(child.stdout);
    
    return {value: object};
}
