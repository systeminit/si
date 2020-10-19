import { PropBool, PropText, PropSelect } from "../../components/prelude";
import { registry } from "../../registry";
import {
  ActionRequest,
  ActionReply,
  ResourceHealth,
  ResourceStatus,
} from "../../veritech/intelligence";

registry.componentAndEntity({
  typeName: "application",
  displayTypeName: "A System Initiative Application",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.integrationServices.push({
      integrationName: "global",
      integrationServiceName: "core",
    });

    c.properties.addSelect({
      name: "inSystems",
      label: "In Systems",
      options(p: PropSelect) {
        p.optionsFromType = "system";
        p.repeated = true;
      },
    });

    c.entity.intelligence.actions = {
      async deploy(request: ActionRequest): Promise<ActionReply> {
        const actions: ActionReply["actions"] = [];
        for (const child of request.successors) {
          if (child.entity.objectType == "service") {
            actions.push({ action: "deploy", entityId: child.entity.id });
          }
        }
        const reply: ActionReply = {
          resource: {
            state: { edward: "van halen" },
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
