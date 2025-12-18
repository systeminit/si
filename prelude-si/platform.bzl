def get_host_platform():
    """Get host platform os and arch for artifact metadata.

    Note: This detects the build host platform. For cross-compilation,
    some artifacts use toolchain-provided platform info instead.

    Returns: tuple of (os, arch) as strings matching artifact format
    """
    os = host_info().os
    arch = host_info().arch

    if os.is_linux:
        os_str = "linux"
    elif os.is_macos:
        os_str = "macos"
    else:
        fail("Unsupported host OS artifact")

    if arch.is_x86_64:
        arch_str = "x86_64"
    elif arch.is_aarch64:
        arch_str = "aarch64"
    else:
        fail("Unsupported host architecture for artifact")

    return (os_str, arch_str)
