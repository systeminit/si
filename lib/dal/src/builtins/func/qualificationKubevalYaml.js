async function qualificationKubevalYaml(input) {
    const code = input.code?.["si:generateYAML"]?.code;
    // This qualification doesn't work because `const file = path.join(...)` throws:
    //   ReferenceError: path is not defined
    if (!code || true) {
        return {
            qualified: false,
            message: "YAML not found for component: " + JSON.stringify(component),
        };
    }
    const file = path.join(os.tmpdir(), makeid(8) + ".yaml");
    fs.writeFileSync(file, code);

    try {
        const child = await siExec.waitUntilEnd("kubeval", [file]);

        return {
            qualified: child.exitCode === 0,
            message: child.stdout + "\n\n" + child.stderr,
        };
    } finally {
        // We might want to garbage collect the tmp folder if the machine we are on isn't rebooted with certain frequency
        // as we might fail to delete some files
        fs.unlinkSync(file);
    }
}

function makeid(length) {
    var result = '';
    var characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    var charactersLength = characters.length;
    for (var i = 0; i < length; i++) {
        result += characters.charAt(Math.floor(Math.random() *
            charactersLength));
    }
    return result;
}

