async function main(input: Input): Promise<Output> {
  // Copied from the region asset
  if (!input.domain?.region) {
    return {
      result: "failure",
      message: "No Region Name to validate",
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-regions",
    "--region-names",
    input.domain?.region!,
    "--region",
    "us-east-1",
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      result: "failure",
      message: "Error from API",
    };
  }

  const regionDetails = JSON.parse(child.stdout).Regions;
  if (regionDetails.length === 0 || regionDetails.length > 1) {
    return {
      result: "failure",
      message: "Unable to find Region",
    };
  }

  if (regionDetails[0].OptInStatus === "not-opted-in") {
    return {
      result: "failure",
      message: "Region not-opted-in for use",
    };
  }

  return {
    result: "success",
    message: "Region is available to use",
  };
}
