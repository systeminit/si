async function qualificationDockerImageNameInspect(component) {
  // TODO: this only works for public images, we need to handle secrets to expand on this
  // TODO: how to ensure skopeo is always installed and in the path? (we added it to the bootstrap script, but is it enough?)
  const child = await siExec.waitUntilEnd("skopeo", ["inspect", `docker://docker.io/${component.data.properties.image}`]);
  return {
    qualified: child.exitCode === 0,
    // Note: Do we want stdout on success? Do we want both, always? Do we want to filter the output?
    message: child.exitCode == 0 ? child.stdout : child.stderr,
  };
}
