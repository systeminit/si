import {
  PropEnum,
  PropObject,
  PropMethod,
  PropLink,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "changeSetEntry",
  displayTypeName: "An entry inside of a change set",
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
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "changeSetId"],
      typeName: "changeSet",
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
        p.properties.addText({
          name: "changeSetId",
          label: "Change Set ID",
          options(p) {
            p.required = true;
          },
        });
      },
    });

    c.fields.addText({
      name: "createdByUserId",
      label: "User ID who created this Change Set Entry",
      options(p) {
        p.required = true;
      },
    });

    c.fields.addText({
      name: "itemId",
      label: "The ID of the item this Change Set Entry points at",
      options(p) {
        p.required = true;
      },
    });

    c.fields.addEnum({
      name: "status",
      label: "The status of this Change Set Entry",
      options(p: PropEnum) {
        p.variants = ["open", "closed", "abandoned"];
        p.baseDefaultValue = "open";
      },
    });

    c.addListMethod();
    c.addGetMethod();

    c.methods.addMethod({
      name: "create",
      label: "Create a Change Set Entry",
      options(p: PropMethod) {
        p.mutation = true;
        p.hidden = true;
        p.skip = true;
        p.isPrivate = true;

        p.request.properties.addText({
          name: "name",
          label: "Name of the changeset entry",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Change Set entry display name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "The SI Properties for this Change Set Entry",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "changeSetEntry",
              names: ["siProperties"],
            };
          },
        });
        p.request.properties.addText({
          name: "createdByUserId",
          label: "User ID who created this Change Set Entry",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${c.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "changeSetEntry",
            };
          },
        });
      },
    });
  },
});
