import { PropText, PropAction } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "helmRepo",
  displayTypeName: "Helm Repo",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c) {
    c.entity.iEntity = {
      uiVisible: true,
      uiMenuCategory: "helm",
      uiMenuDisplayName: "repo",
    };

    c.entity.inputType("helmRepoCredential");
    c.entity.inputType("awsAccessKeyCredential");
    c.entity.inputType("aws");
    c.entity.inputType("awsEks");

    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });

    c.properties.addText({
      name: "name",
      label: "Repo Name",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.properties.addText({
      name: "url",
      label: "Repo Name",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.properties.addBool({
      name: "insecureSkipTlsVerify",
      label: "Insecure Skip TLS Verify",
    });
  },
});
