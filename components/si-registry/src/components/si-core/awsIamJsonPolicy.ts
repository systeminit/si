import { PropCode, PropTextArea } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "awsIamJsonPolicy",
  displayTypeName: "AWS IAM JSON policy",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.inputType("awsAccessKeyCredential");

    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });

    // Properties
    c.properties.addTextArea({
      name: "object",
      label: "Object",
      options(p: PropTextArea) {
        p.required = true;

        p.relationships.updates({
          partner: {
            typeName: "awsIamJsonPolicy",
            names: ["properties", "objectJson"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "awsIamJsonPolicy",
            names: ["properties", "objectJson"],
          },
        });
      },
    });
    c.properties.addCode({
      name: "objectJson",
      label: "Object JSON",
      options(p: PropCode) {
        p.relationships.updates({
          partner: {
            typeName: "awsIamJsonPolicy",
            names: ["properties", "object"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "awsIamJsonPolicy",
            names: ["properties", "object"],
          },
        });
        p.language = "json";
      },
    });

    // Entity Actions
    c.entity.methods.addAction({
      name: "create",
      label: "Create Policy",
    });
  },
});
