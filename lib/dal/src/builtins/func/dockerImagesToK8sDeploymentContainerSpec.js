function dockerImagesToK8sDeploymentContainerSpec(component) {
    let result = [];
    let images = component.data.properties.images;
    // Force the images arg to be an Array (and return an empty array if the arg is absent/undefined/null).
    if (images === undefined) return result;
    if (images === null) return result;
    if (!Array.isArray(images)) images = [images];

    images.forEach(function (dockerImage) {
        let deploymentContainer = {};
        deploymentContainer.image = dockerImage.image;
        deploymentContainer.ports = [];
        let exposedPorts = dockerImage.ExposedPorts;
        if (!(exposedPorts === undefined || exposedPorts === null)) {
            exposedPorts.forEach(function (exposedPort) {
                if (!(exposedPort === undefined || exposedPorts === null)) {
                    let parts = exposedPort.split('/');
                    try {
                      let containerPort = {};
                      containerPort.containerPort = parseInt(parts[0]);
                      if (parts.length === 1) {
                        containerPort.protocol = "TCP";
                      } else if (parts.length === 2) {
                        containerPort.protocol = parts[1].toUpperCase();
                      }
                      deploymentContainer.ports.push(containerPort);
                    } catch (err) {}
                }
            });
        }

        result.push(deploymentContainer);
    });

    return result;
}
