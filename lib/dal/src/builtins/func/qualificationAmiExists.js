async function qualification(component) {
  const {
    data: {
      properties: {
        domain: {
          region,
          ImageId
        }
      }
    }
  } = component

  if (!region) {
    return {
      qualified: false,
      message: "Component doesn't have a region set"
    }
  }

  const dryRunStatus = await siExec.waitUntilEnd("aws", ["ec2", "describe-images",
    "--region", region,
    "--filters", `Name=image-id,Values=${ImageId}`])

  console.log(dryRunStatus.stderr);

  if (dryRunStatus.exitCode !== 0) {
    return {
      qualified: false,
      message: dryRunStatus.shortMessage
    }
  }

  const { Images: images } = JSON.parse(dryRunStatus.stdout)

  const success = images.length === 1;

  return {
    qualified: success,
    message: success ? 'Image exists' : "Image not found on region"
  }
}
