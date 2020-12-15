import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "helmRepoCredential",
  displayTypeName: "Helm Repo Credential",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.iEntity = {
      uiVisible: true,
      uiMenuCategory: "helm",
      uiMenuDisplayName: "repoCredential",
    };

    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });
    c.entity.secretType("credential", "helmRepo");
    c.properties.addText({
      name: "username",
      label: "Username",
    });
    c.properties.addText({
      name: "password",
      label: "Password",
    });
  },
});
