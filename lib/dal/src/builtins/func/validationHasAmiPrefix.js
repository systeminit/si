function hasAmiPrefix(input) {
    const ami = input.value ?? '';
    const valid = typeof ami === 'string' && ami.startsWith('ami-');
    return {
        valid,
        message: valid ? undefined : `Image id '${ami}' must start with 'ami-'` 
    };
}