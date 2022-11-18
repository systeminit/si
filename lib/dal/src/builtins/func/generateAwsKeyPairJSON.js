async function generateAwsKeyPairJSON(input) {
    // Initialize the input JSON.
    const object = {
        "KeyName": input.domain.KeyName,
        "KeyType": input.domain.KeyType,
    };

    // Normalize tags to be in the weird Map-like structure AWS uses (array of { Key: string, Value: string } where Key is unique
    const tags = [];
    if (input.domain.tags) {
        for (const [key, value] of Object.entries(input.domain.tags)) {
            tags.push({
                "Key": key,
                "Value": value,
            });
        }
        if (tags.length > 0) {
            object["TagSpecifications"] = [{
                "ResourceType": input.domain.awsResourceType,
                "Tags": tags
            }];
        }
    }

    // FIXME(nick): once the bug related to child fields for complex objects is fixed, return the format too.
    // return {
    //     format: "json",
    //     code: JSON.stringify(object, null, '\t'),
    // };
    return JSON.stringify(object, null, '\t');
}
