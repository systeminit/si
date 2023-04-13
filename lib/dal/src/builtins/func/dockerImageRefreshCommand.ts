async function refresh(component: Input): Promise<Output> {
  const child = await siExec.waitUntilEnd("skopeo", ["inspect", "--override-os", "linux", "--override-arch", "amd64", `docker://docker.io/${component.properties.domain.image}`]);
  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message: "Unable to refresh docker image",
      value: component.properties.resource?.value,
    }
  }

  return { value: JSON.parse(child.stdout), status: "ok" };
}
