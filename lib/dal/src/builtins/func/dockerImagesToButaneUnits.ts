async function dockerImagesToButaneUnits(input: Input): Promise<Output> {
  let units = [];
  let images = input.images;
  // Force the images arg to be an Array (and return an empty array if the arg is absent/undefined/null).
  if (images === undefined) return units;
  if (images === null) return units;
  if (!Array.isArray(images)) images = [images];

  images.filter(i => i ?? false).forEach(function (dockerImage) {

    // Only allow "valid DNS characters" for the container name, and make sure it doesn't
    // end with a dash character ("-").
    let name = dockerImage.si.name
      .replace(/[^A-Za-z0-9]/g, '-')
      .replace(/-+$/, '')
      .toLowerCase();
    let unit = {
      name: name + ".service",
      enabled: true
    };

    let ports = "";
    let dockerImageExposedPorts = dockerImage.domain.ExposedPorts;
    if (!(dockerImageExposedPorts === undefined || dockerImageExposedPorts === null)) {
      dockerImageExposedPorts.forEach(function (dockerImageExposedPort) {
        if (!(dockerImageExposedPort === undefined || dockerImageExposedPort === null)) {
          let parts = dockerImageExposedPort.split('/');
          try {
            // Prefix with a blank space.
            ports = ports + ` --publish ${parts[0]}:${parts[0]}`;
          } catch (err) {
          }
        }
      });
    }

    let image = dockerImage.domain.image;
    let defaultDockerHost = "docker.io";
    let imageParts = image.split("/");
    if (imageParts.length === 1) {
      image = [defaultDockerHost, "library", imageParts[0]].join("/");
    } else if (imageParts.length === 2) {
      image = [defaultDockerHost, imageParts[0], imageParts[1]].join("/");
    }

    let description = name.charAt(0).toUpperCase() + name.slice(1);

    // Ensure there is no space between "name" and "ports" as ports are optional.
    unit.contents = `[Unit]\nDescription=${description}\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill ${name}\nExecStartPre=-/bin/podman rm ${name}\nExecStartPre=/bin/podman pull ${image}\nExecStart=/bin/podman run --name ${name}${ports} ${image}\n\n[Install]\nWantedBy=multi-user.target`;

    units.push(unit);
  });

  return units;
}
