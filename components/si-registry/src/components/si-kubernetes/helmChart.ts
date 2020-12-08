import { PropText, PropAction } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "helmChart",
  displayTypeName: "Helm Chart",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c) {
    c.entity.inputType("helmRepoCredential");
    c.entity.inputType("awsAccessKeyCredential");
    c.entity.inputType("aws");
    c.entity.inputType("awsEks");
    c.entity.inputType("helmRepo");

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
      label: "Chart Name",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.properties.addText({
      name: "version",
      label: "Chart Version",
    });

    c.properties.addBool({
      name: "insecureSkipTlsVerify",
      label: "Insecure Skip TLS Verify",
    });
  },
});
