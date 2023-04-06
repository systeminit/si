async function extract(input) {
  if (!input.code) {
    return ""
  }

    const code = input.code?.["si:generateAwsAmiJSON"]?.code;
    if (!code) {
      return ""
    }
    // Ensure we have filters (an ami id or filters are set)
    const filters = JSON.parse(code)?.Filters ?? [];
    if (filters.length == 0) {
      return "";
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
  ])

  if (imageResponse.exitCode !== 0) {
    console.log("error returned from AWS API")
    return ""
  }

  const images = JSON.parse(imageResponse.stdout)
  if (images.length === 1 || (images.length > 1 && input.domain.UseMostRecent)) {
    return images[0].ImageId
  }

  console.log("Unable to find a suitable ImageID")
  return "";
}
