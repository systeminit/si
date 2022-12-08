function generateAwsAmiJSON(input) {
    // Initialize the input JSON.
    const object = {
        "ImageIds": [input.domain.ImageId],
    };

    return {
        format: "json",
        code: JSON.stringify(object, null, '\t')
    };
}
