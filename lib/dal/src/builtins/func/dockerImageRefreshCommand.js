async function refresh(component) {
  const child = await siExec.waitUntilEnd("skopeo", ["inspect", "--override-os", "linux", "--override-arch", "amd64", `docker://docker.io/${component.properties.domain.image}`]);
  if (child.exitCode !== 0) {
    throw new Error(child.stderr);
  }

  return { value: JSON.parse(child.stdout) };
}
