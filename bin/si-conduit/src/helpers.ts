
export function unknownValueToErrorMessage(value: unknown): string {
    if (typeof value === 'string') return value;

    if (value instanceof Error) return value.message;

    return `Unknown Error: ${value}`;
}

export function makeStringSafeForFilename(str: string): string {
    return str.replace(/[\\/:*?"<>|]/g, '_');
}
