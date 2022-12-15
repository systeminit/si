async function qualification(input) {
    const code = input.code?.["si:generateAwsEc2JSON"]?.code;
    if (!code) {
        return {
            qualified: false,
            message: "component doesn't have JSON representation"
        }
    }

    if (!input.domain.region) {
        return {
            qualified: false,
            message: "component doesn't have a region set"
        }
    }

    const dryRunStatus = await siExec.waitUntilEnd("aws", [
        "ec2",
        "run-instances",
        "--region",
        input.domain.region,
        "--dry-run",
        "--cli-input-json",
        code
    ]);

    console.log(dryRunStatus.stderr);

    // We have to use `includes` instead of `startsWith` because the line can start with a line feed char
    const success = dryRunStatus.stderr.includes('An error occurred (DryRunOperation)');

    return {
        qualified: success,
        message: success ? 'component qualified' : dryRunStatus.shortMessage
    }
}
