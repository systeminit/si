async function qualificationDockerImageNameInspect(component) {
  console.log(JSON.stringify(component))
  const child = await siExec.waitUntilEnd("skopeo", ["inspect", "--override-os", "linux", "--override-arch", "amd64", `docker://docker.io/${component.data.properties.domain.image}`]);
  return {
    qualified: child.exitCode === 0,
    // Note: Do we want stdout on success? Do we want both, always? Do we want to filter the output?
    message: child.exitCode == 0 ? child.stdout : child.stderr,
  };
}
