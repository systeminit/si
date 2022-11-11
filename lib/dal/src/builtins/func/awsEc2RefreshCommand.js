async function refresh(component) {
    const resource = component.properties.resource;
    if (!resource || resource === "null") return null;
    const instances = Array.isArray(resource) ? resource : [resource];
    const instanceIds = instances.flatMap((i) => i.Instances).map((i) => i.InstanceId).filter((id) => !!id);
    if (!instanceIds || instanceIds.length === 0) return null;

    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "describe-instances",
	"--instance-ids",
	...instanceIds
    ]);

    if (child.exitCode !== 0) {
        throw new Error(`Failure running aws ec2 describe-instances (${child.exitCode}): ${child.stderr}`);
    }

    console.log(child.stdout);
    return { value: JSON.parse(child.stdout).Reservations };
}
