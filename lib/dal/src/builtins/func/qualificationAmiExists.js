async function qualification(input) {
    const {
        domain: {
            region,
            ImageId
        }
    } = input

    if (!region) {
        return {
            qualified: false,
            message: "region is unset"
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

    const {Images: images} = JSON.parse(dryRunStatus.stdout)

    const success = images.length === 1;

    return {
        qualified: success,
        message: success ? 'Image exists' : "Image not found on region"
    }
}
