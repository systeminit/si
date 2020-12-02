import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "awsAccessKeyCredential",
  displayTypeName: "AWS Access Key Credential",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.iEntity = {
      uiVisible: true,
      uiMenuCategory: "aws",
      uiMenuDisplayName: "access key"
    };
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });
    c.entity.secretType("credential", "awsAccessKey");
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
