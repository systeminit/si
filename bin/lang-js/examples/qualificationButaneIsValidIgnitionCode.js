async function qualificationButaneIsValidIgnition(_component) {
    const domainJson = "{\"systemd\":{\"units\":[{\"name\":\"whiskers.service\",\"enabled\":true,\"contents\":\"[Unit]\\nDescription=Whiskers\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill whiskers1\\nExecStartPre=-/bin/podman rm whiskers1\\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\\nExecStart=/bin/podman run --name whiskers1 --publish 80:80 docker.io/systeminit/whiskers\\n\\n[Install]\\nWantedBy=multi-user.target\"}]},\"variant\":\"fcos\",\"version\":\"1.4.0\"}";
    domainJson.replace("\n", "\\\\n");
    const options = {input: `${domainJson}`};
    const child = await siExec.waitUntilEnd("butane", ["--pretty", "--strict"], options);
    if (child.exitCode === 0) console.log(child.stdout);
    return {
        qualified: child.exitCode === 0,
        message: child.exitCode === 0 ? child.stdout : child.stderr,
    };
}

