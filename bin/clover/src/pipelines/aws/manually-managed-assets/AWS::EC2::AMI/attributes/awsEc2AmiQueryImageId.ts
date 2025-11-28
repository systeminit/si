async function main({ region, Filters, ExecutableUsers, Owners }: Input): Promise < Output > {
  async function awsEc2DescribeImages(region: string, queryArgs: string[]) {
    const response = await siExec.waitUntilEnd("aws", [
      "ec2",
      "describe-images",
      "--region",
      region ?? "",
      "--query",
      "reverse(sort_by(Images, &CreationDate))",
      ...queryArgs
    ]);
    if (response.exitCode !== 0) { throw new Error(`aws command failed with exit code ${response.exitCode}. Output:\n${response.all}`); }
    const images = JSON.parse(response.stdout) as { ImageId: string }[];
    return images.map(image => image.ImageId);
  }

  const queryArgs = [];
  for (const f of Filters ?? []) {
    queryArgs.push("--filters", `Name=${f.Name},Values=${f.Value}`);
  }
  if (ExecutableUsers) queryArgs.push("--executable-users", ExecutableUsers);
  if (Owners) queryArgs.push("--owners", Owners);

  // If no query is specified, there's nothing to return
  if (queryArgs.length == 0) {
    return "";
  }

  // Get the image back from the query
  const imageIds = await awsEc2DescribeImages(region, queryArgs);
  return imageIds[0] ?? "";
}
