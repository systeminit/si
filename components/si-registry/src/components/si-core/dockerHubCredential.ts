import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "dockerHubCredential",
  displayTypeName: "Docker Hub Credential",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.iEntity = {
      uiVisible: true,
      uiMenuCategory: "docker",
      uiMenuDisplayName: "credential"
    };
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });
    c.entity.secretType("credential", "dockerHub");
    c.properties.addText({
      name: "secretId",
      label: "Secret Id",
    });
    c.properties.addText({
      name: "secretName",
      label: "Secret Name",
    });
  },
});
