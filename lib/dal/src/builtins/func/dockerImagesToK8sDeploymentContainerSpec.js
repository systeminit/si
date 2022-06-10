function dockerImagesToK8sDeploymentContainerSpec(args) {
    let result = [];
    let images = args.images;
    // Force the images arg to be an Array (and return an empty array if the arg is absent/undefined/null).
    if (images === undefined) return result;
    if (images === null) return result;
    if (!Array.isArray(images) images = [images];

    images.forEach(function (dockerImage) {
        let deploymentContainer = {};
        deploymentContainer.image = dockerImage.image;
        deploymentContainer.containerPorts = [];
        let exposedPorts = dockerImage.exposedPorts;
        if (!(exposedPorts === undefined || exposedPorts === null)) {
            deploymentContainer.containerPorts = [];
            exposedPorts.forEach(function (exposedPort) {
                if (!(exposedPort === undefined || exposedPorts === null)) {
                    let parts = exposedPort.split('/');
                    let containerPort = {};
                    containerPort.port = parts[0];
                    containerPort.protocol = parts[1].toUpperCase();
                    deploymentContainer.containerPorts.push(containerPort);
                }
            });
        }

        result.push(deploymentContainer);
    });

    return result;
}
