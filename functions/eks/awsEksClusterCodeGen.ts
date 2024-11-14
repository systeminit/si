async function main({ domain }: Input): Promise<Output> {
    // Steal fields from input.domain and assign the rest to "code"
    const { extra, enabledLoggingTypes, encryptionConfig, ...code } = domain ?? {};

    if (enabledLoggingTypes?.length > 0) {
        code.logging = { clusterLogging: [{ types: enabledLoggingTypes, enabled: true }] }
    }

    if (encryptionConfig?.resources?.length > 0) {
        code.encryptionConfig = [encryptionConfig]
    }

    return { format: "json", code };
}