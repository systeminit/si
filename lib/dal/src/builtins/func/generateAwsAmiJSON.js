function generateAwsAmiJSON(input) {
    // Initialize the input JSON.
    const object = {
        "ImageIds": [input.domain.ImageId],
    };

    // FIXME(nick): once the bug related to child fields for complex objects is fixed, return the format too.
    // return {
    //     format: "json",
    //     code: JSON.stringify(object, null, '\t')
    // };
    return JSON.stringify(object, null, '\t');
}
