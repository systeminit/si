def alias(
        visibility = ["PUBLIC"],
        **kwargs):
    native.alias(visibility = visibility, **kwargs)

def export_file(
        visibility = ["PUBLIC"],
        **kwargs):
    native.export_file(visibility = visibility, **kwargs)

def filegroup(
        visibility = ["PUBLIC"],
        **kwargs):
    native.filegroup(visibility = visibility, **kwargs)

def sh_binary(
        visibility = ["PUBLIC"],
        **kwargs):
    native.sh_binary(visibility = visibility, **kwargs)
