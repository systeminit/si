import {
  PropEnum,
  PropObject,
  PropMethod,
  PropLink,
  PropNumber,
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
    c.associations.hasMany({
      fieldName: "changeSetEntries",
      typeName: "item",
      queryField: "siStorable.changeSetId",
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
    c.fields.addNumber({
      name: "entryCount",
      label: "Entry Count",
      options(p: PropNumber) {
        p.numberKind = "uint64";
        p.baseDefaultValue = "0";
      },
    });
    c.fields.addEnum({
      name: "status",
      label: "The status of this Change Set",
      options(p: PropEnum) {
        p.variants = ["open", "closed", "abandoned", "executing", "failed"];
        p.baseDefaultValue = "open";
      },
    });
    c.fields.addText({
      name: "note",
      label: "Note",
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
          label: "Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Display Name",
        });
        p.request.properties.addText({
          name: "note",
          label: "Note",
        });
        p.request.properties.addText({
          name: "workspaceId",
          label: `Workspace ID`,
          options(p) {
            p.required = true;
            p.hidden = true;
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

    c.methods.addMethod({
      name: "execute",
      label: "Execute a Change Set",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "id",
          label: "Change Set ID",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `ChangeSet Item`,
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
