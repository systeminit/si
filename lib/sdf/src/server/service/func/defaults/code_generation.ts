function generateCode(component) {
    return {
        format: "json",
        code: JSON.stringify(component.properties),
    };
}