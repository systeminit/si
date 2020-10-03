import {
  PropObject,
  PropNumber,
  PropMethod,
  PropLink,
  PropEnum,
  PropText,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "eventLog",
  displayTypeName: "Event Log Entry",
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
    c.fields.addEnum({
      name: "level",
      label: "Event Level",
      options(p: PropEnum) {
        p.variants = ["trace", "debug", "info", "warn", "error"];
      },
    });
    c.fields.addText({
      name: "message",
      label: "Message",
    });
    c.fields.addObject({
      name: "payload",
      label: "Structured Event Payload",
      options(p: PropObject) {
        p.required = true;
        p.properties.addText({
          name: "kind",
          label: "Type of Event",
        });
        p.properties.addText({
          name: "data",
          label: "JSON data",
        });
      },
    });
    c.fields.addText({
      name: "relatedIds",
      label: "Related IDs for this Event",
      options(p: PropText) {
        p.repeated = true;
      },
    });
    c.fields.addText({
      name: "timestamp",
      label: "The timestamp for this event",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.fields.addText({
      name: "createdByUserId",
      label: "User ID who created this Change Set",
      options(p) {
        p.required = true;
      },
    });

    c.addListMethod();
    c.addGetMethod();

    c.methods.addMethod({
      name: "create",
      label: "Create an Event Log Entry",
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
        p.request.properties.addLink({
          name: "level",
          label: "Event Level",
          options(p: PropLink) {
            p.lookup = {
              typeName: "eventLog",
              names: ["level"],
            };
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "Si Properties",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "eventLog",
              names: ["siProperties"],
            };
          },
        });
        p.request.properties.addLink({
          name: "message",
          label: "Message",
          options(p: PropLink) {
            p.lookup = {
              typeName: "eventLog",
              names: ["message"],
            };
          },
        });
        p.request.properties.addLink({
          name: "payload",
          label: "Structured Event Payload",
          options(p: PropLink) {
            p.lookup = {
              typeName: "eventLog",
              names: ["payload"],
            };
          },
        });
        p.request.properties.addLink({
          name: "relatedIds",
          label: "Related IDs for this Event",
          options(p: PropLink) {
            p.repeated = true;
            p.lookup = {
              typeName: "eventLog",
              names: ["relatedIds"],
            };
          },
        });
        p.request.properties.addLink({
          name: "timestamp",
          label: "Timestamp this Event",
          options(p: PropLink) {
            p.lookup = {
              typeName: "eventLog",
              names: ["timestamp"],
            };
          },
        });
        p.request.properties.addLink({
          name: "createdByUserId",
          label: "User ID for this event",
          options(p: PropLink) {
            p.lookup = {
              typeName: "eventLog",
              names: ["createdByUserId"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${c.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "eventLog",
            };
          },
        });
      },
    });
  },
});
