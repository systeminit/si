function isValidHexColor(input) {
    const color = input.value;
    const regex = new RegExp('^#[\dA-Fa-f]{6}$');
    const isValidHex = regex.test(color);
    return {
        valid: isValidHex,
        message: isValidHex ? undefined : `'${color}' must be a valid hex color triplet` 
    };
}