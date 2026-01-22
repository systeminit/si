async function main({
    properties,
    extra
}: Input): Promise<Output> {
    if (!properties) {
        return null;
    }
    return JSON.stringify({
        Type: extra?.AwsResourceType,
        Properties: properties
    }, null, 2);
}
