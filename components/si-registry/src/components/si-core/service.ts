import {
  PropAction,
  PropNumber,
  PropText,
  PropSelect,
} from "../../components/prelude";
import {
  ActionRequest,
  ActionReply,
  ResourceHealth,
  ResourceStatus,
} from "../../veritech/intelligence";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "service",
  displayTypeName: "Service",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.entity.integrationServices.push({
      integrationName: "global",
      integrationServiceName: "core",
    });

    // Properties
    c.properties.addText({
      name: "image",
      label: "Container Image",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.properties.addNumber({
      name: "port",
      label: "Container Port",
      options(p: PropNumber) {
        p.required = true;
        p.numberKind = "uint32";
      },
    });
    c.properties.addNumber({
      name: "replicas",
      label: "Replicas",
      options(p: PropNumber) {
        p.numberKind = "uint32";
        p.baseDefaultValue = "1";
      },
    });
    c.properties.addSelect({
      name: "deploymentTarget",
      label: "Deployment Target",
      options(p: PropSelect) {
        p.baseValidation = p.baseValidation.allow(
          "none",
          "kubernetes-minikube",
          "kubernetes-aws",
        );
        p.options = [
          { key: "none", value: "none" },
          { key: "kubernetes", value: "kubernetes" },
          { key: "docker", value: "docker" },
        ];
      },
    });

    // Entity Actions
    c.entity.methods.addAction({
      name: "deploy",
      label: "Deploy",
      options(p: PropAction) {
        p.mutation = true;
      },
    });

    c.entity.intelligence.actions = {
      async deploy(request: ActionRequest): Promise<ActionReply> {
        const actions: ActionReply["actions"] = [];
        for (const child of request.entities.successors) {
          if (child.objectType == "service") {
            actions.push({ action: "deploy", nodeId: child.nodeId });
          }
        }
        const reply: ActionReply = {
          resource: {
            state: {
              alex: "van halen",
              deployedBy: request.entities.predecessors.map(p => p.name),
            },
            health: ResourceHealth.Ok,
            status: ResourceStatus.Created,
          },
          actions,
        };
        return reply;
      },
    };
  },
});
