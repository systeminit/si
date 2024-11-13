async function main(input: Input): Promise<Output> {
    if (input.domain?.extra) {
        delete input.domain.extra;
    }

    const object = input.domain;

    if (input.domain?.enabledLoggingTypes) {
        let loggingTypes: String[] = [];
        for (const loggingType of input.domain?.enabledLoggingTypes) {
            loggingTypes.push(loggingType)
        }
        object.logging = {
            clusterLogging: [{
                types: loggingTypes,
                enabled: true,
            }]
        }
        delete object.enabledLoggingTypes
    }

    if (input.domain.encryptionConfig) {
        if (input.domain.encryptionConfig.resources && input.domain.encryptionConfig.resources.length > 0) {
            let config: Record<string, any> = {
                resources: input.domain.encryptionConfig.resources,
                provider: {
                    KeyArn: input.domain.encryptionConfig.provider.keyArn
                }
            }
            object.encryptionConfig = [config];
        } else {
            delete object.encryptionConfig
        }
    }

    return {
        format: "json",
        code: JSON.stringify(object || {}, null, 2),
    };
}