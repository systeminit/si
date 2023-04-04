async function qualificationDockerImageExists(input) {
    if (!input.domain?.image) {
        return {
            result: "failure",
            message: "no image available"
        }
    }
    const child = await siExec.waitUntilEnd("skopeo", ["inspect", "--override-os", "linux", "--override-arch", "amd64", `docker://${input.domain.image}`]);
    return {
        result: child.exitCode === 0 ? "success" : "failure",
        // Note: Do we want stdout on success? Do we want both, always? Do we want to filter the output?
        message: child.exitCode === 0 ? child.stdout : child.stderr,
    };
}
