async function qualificationDockerImageExists(
  component: Input,
): Promise<Output> {
  if (!component.domain?.image) {
    return {
      result: "failure",
      message: "no image available",
    };
  }
  const child = await siExec.waitUntilEnd("skopeo", [
    "inspect",
    "--override-os",
    "linux",
    "--override-arch",
    "amd64",
    `docker://${component.domain.image}`,
  ]);
  return {
    result: child.exitCode === 0 ? "success" : "failure",
    message: child.exitCode === 0 ? child.stdout : child.stderr,
  };
}
