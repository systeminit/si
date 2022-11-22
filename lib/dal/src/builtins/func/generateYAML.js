function generateYAML(input) {
    // FIXME(nick): once the bug related to child fields for complex objects is fixed, return the format too.
    // return {
    //     format: "yaml",
    //     code: Object.keys(input.domain).length > 0 ?
    //         YAML.stringify(input.domain) : ""
    // };
    return Object.keys(input.domain).length > 0 ?
        YAML.stringify(input.domain) : "";
}