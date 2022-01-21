function dockerImageNameEqualsComponentName(component) {
    const result = component.name === component.properties.image;
    if (result) {
        return { qualified: result, message: "dockerImageName is not equal to componentName" }
    }
    return { qualified: result }
}