async function qualification(input) {
    const {
        domain: {
            region,
            ImageId
        }
    } = input

    if (!region) {
        return {
            result: "failure",
            message: "region is unset"
        }
    }

    const dryRunStatus = await siExec.waitUntilEnd("aws", ["ec2", "describe-images",
        "--region", region,
        "--filters", `Name=image-id,Values=${ImageId}`])

    console.log(dryRunStatus.stderr);

    if (dryRunStatus.exitCode !== 0) {
        return {
            result: "failure",
            message: dryRunStatus.shortMessage
        }
    }

    const {Images: images} = JSON.parse(dryRunStatus.stdout)

    const success = images.length === 1;

    return {
        result: success ? "success" : "failure",
        message: success ? 'Image exists' : "Image not found on region"
    }
}
