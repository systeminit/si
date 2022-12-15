async function qualification(input) {
    const code = input.code?.["si:generateAwsKeyPairJSON"]?.code;
    if (!code) {
        return {
            qualified: false,
            message: "component doesn't have JSON representation"
        }
    }

    if (!input.domain?.region) {
        return {
            qualified: false,
            message: "component doesn't have a region set"
        }
    }

    // Dry run creation of the key pair.
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "create-key-pair",
        "--region",
        input.domain.region,
        "--dry-run",
        "--cli-input-json",
        code,
    ]);

    // We have to use `includes` instead of `startsWith` because the line can start with a line feed char
    const success = child.stderr.includes('An error occurred (DryRunOperation)');

    return {
        qualified: success,
        message: success ? 'component qualified' : child.stderr,
    }
}
