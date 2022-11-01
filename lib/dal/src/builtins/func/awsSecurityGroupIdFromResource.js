function parse(properties) {
    if (!properties.resource) return "";
    const obj = JSON.parse(properties.resource);
    return obj["GroupId"] ?? "";
}
