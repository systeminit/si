import {
  PropEnum,
  PropObject,
  PropMethod,
  PropLink,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "changeSet",
  displayTypeName: "A change set for your system",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "organizationId"],
      typeName: "organization",
    });
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "workspaceId"],
      typeName: "workspace",
    });

    c.fields.addObject({
      name: "siProperties",
      label: "SI Internal Properties",
      options(p: PropObject) {
        p.required = true;
        p.properties.addText({
          name: "billingAccountId",
          label: "Billing Account ID",
          options(p) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "organizationId",
          label: "Organization ID",
          options(p) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "workspaceId",
          label: "Organization ID",
          options(p) {
            p.required = true;
          },
        });
      },
    });

    c.fields.addText({
      name: "createdByUserId",
      label: "User ID who created this Change Set",
      options(p) {
        p.required = true;
      },
    });
    c.fields.addText({
      name: "changeSetItemOrder",
      label: "The list of change set items, in order",
      options(p) {
        p.repeated = true;
      },
    });
    c.fields.addEnum({
      name: "status",
      label: "The status of this Change Set",
      options(p: PropEnum) {
        p.variants = ["open", "closed", "abandoned"];
        p.baseDefaultValue = "open";
      },
    });

    c.addListMethod();
    c.addGetMethod();

    c.methods.addMethod({
      name: "create",
      label: "Create a Change Set",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "name",
          label: "Name of the changeset",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Change Set display name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "The SI Properties for this User",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "changeSet",
              names: ["siProperties"],
            };
          },
        });
        p.request.properties.addText({
          name: "createdByUserId",
          label: "User ID who created this Change Set",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${c.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "changeSet",
            };
          },
        });
      },
    });
  },
});
