import { PropText, PropAction } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "helmRelease",
  displayTypeName: "Helm Release",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c) {
    c.entity.inputType("helmRepoCredential");
    c.entity.inputType("awsAccessKeyCredential");
    c.entity.inputType("aws");
    c.entity.inputType("awsEks");
    c.entity.inputType("helmRepo");
    c.entity.inputType("helmChart");

    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });

    c.entity.methods.addAction({
      name: "install",
      label: "install",
      options(p: PropAction) {
        p.mutation = true;
      },
    });

    c.entity.methods.addAction({
      name: "upgrade",
      label: "upgrade",
      options(p: PropAction) {
        p.mutation = true;
      },
    });

    c.entity.methods.addAction({
      name: "apply",
      label: "apply",
      options(p: PropAction) {
        p.mutation = true;
      },
    });

    c.properties.addText({
      name: "name",
      label: "Release Name",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.properties.addText({
      name: "description",
      label: "Release Description",
    });

    c.properties.addTextArea({
      name: "valuesYaml",
      label: "Values YAML",
    });

    c.properties.addBool({
      name: "atomic",
      label: "Atomic install",
    });

    c.properties.addBool({
      name: "noHooks",
      label: "Don't run hooks",
    });

    c.properties.addBool({
      name: "renderSubchartNotes",
      label: "Render subchart notes",
    });

    c.properties.addBool({
      name: "skipCrds",
      label: "Skip CRDs",
    });

    c.properties.addBool({
      name: "wait",
      label: "wait for all resources",
    });

    c.properties.addText({
      name: "timeout",
      label: "Timeout for each k8s operation",
    });

    c.properties.addBool({
      name: "insecureSkipTlsVerify",
      label: "Insecure Skip TLS Verify",
    });
  },
});
