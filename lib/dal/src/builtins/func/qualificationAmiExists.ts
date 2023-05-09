async function qualification(input: Input): Promise<Output> {
  const code = input.code?.["si:generateAwsAmiJSON"]?.code;
  if (!code) {
    return {
      result: "failure",
      message: "component doesn't have JSON representation",
    };
  }

  const filtersCode = JSON.parse(code);
  if (filtersCode.Filters?.length == 0) {
    return {
      result: "failure",
      message: "Please ensure that Image ID or a Filter is set",
    };
  }

  if (!input.domain.region) {
    return {
      result: "failure",
      message: "component doesn't have a region set",
    };
  }

  const imageResponse = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-images",
    "--region",
    input.domain.region,
    "--query",
    "reverse(sort_by(Images, &CreationDate))",
    "--cli-input-json",
    code,
  ]);

  if (imageResponse.exitCode !== 0) {
    return {
      result: "failure",
      message: imageResponse.shortMessage,
    };
  }

  const images = JSON.parse(imageResponse.stdout);
  if (images.length === 0) {
    return {
      result: "failure",
      message: "Image not found",
    };
  }

  if (images.length > 1 && !input.domain.UseMostRecent) {
    return {
      result: "failure",
      message:
        "More than 1 AMI returned from the AMI Search. Please narrow the search.",
    };
  }

  return {
    result: "success",
    message: "Image exists",
  };
}
