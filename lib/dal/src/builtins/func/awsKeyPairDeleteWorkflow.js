async function create(arg) {
    return {
        name: "si:awsKeyPairDeleteWorkflow",
        kind: "conditional",
        steps: [
            {
                command: "si:awsKeyPairDeleteCommand",
                args: [arg],
            },
        ],
    };
}
