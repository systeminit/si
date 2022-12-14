function isValidPortNumber(input) {
    const port = input.value;
    const isValidPort = port > 0 && port <= 65536;
    return {
        valid: isValidPort,
        message: isValidPort ? undefined : `'${port}' must be an integer between 1 and 65536` 
    };
}