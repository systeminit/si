async function failure() {
    return {
        name: "si:failure",
        kind: "conditional",
        steps: [
            {
                command: "si:fail",
            },
        ],
    };
}
