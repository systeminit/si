import { PropObject, PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "editSession",
  displayTypeName: "An edit session",
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
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "userId"],
      typeName: "user",
    });
    c.associations.hasMany({
      fieldName: "changeSetEntries",
      typeName: "item",
      queryField: "siStorable.editSessionId",
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
        p.properties.addText({
          name: "userId",
          label: "The user this edit session belongs to",
          options(p) {
            p.required = true;
          },
        });
      },
    });
    c.fields.addText({
      name: "note",
      label: "Node for this edit session",
    });
    c.fields.addBool({
      name: "reverted",
      label: "If this edit session is reverted",
    });
    c.fields.addText({
      name: "changeSetEntryIds",
      label: "List of change set entry ids",
      options(p) {
        p.repeated = true;
      },
    });

    c.addCreateMethod();
    c.addListMethod();
    c.addGetMethod();

    c.methods.addMethod({
      name: "revert",
      label: "Revert an Edit Session",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "id",
          label: "Edit Session ID",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `Edit Session Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "editSession",
            };
          },
        });
      },
    });
    c.methods.addMethod({
      name: "unrevert",
      label: "Un-Revert an Edit Session",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "id",
          label: "Edit Session ID",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `Edit Session Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "editSession",
            };
          },
        });
      },
    });
  },
});
