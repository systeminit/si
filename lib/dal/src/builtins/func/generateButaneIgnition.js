async function generateButaneIgnition(input) {
    const domainJson = JSON.stringify(input.domain);
    domainJson.replace("\n", "\\\\n");
    const options = {input: `${domainJson}`};
    const {stdout} = await siExec.waitUntilEnd("butane", ["--pretty", "--strict"], options);

    // FIXME(nick): once the bug related to child fields for complex objects is fixed, return the format too.
    // return {
    //     format: "json",
    //     code: stdout.toString(),
    // };
    return stdout.toString();
}
