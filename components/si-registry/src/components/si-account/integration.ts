import {
  PropObject,
  PropNumber,
  PropMethod,
  PropLink,
  PropText,
  PropEnum,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "integration",
  displayTypeName: "An integration with another system",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.migrateable = true;

    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.associations.hasMany({
      fieldName: "integrationInstances",
      typeName: "integrationInstance",
    });
    c.fields.addObject({
      name: "options",
      label: "Options for this Integration",
      options(p: PropObject) {
        p.repeated = true;
        p.properties.addText({
          name: "name",
          label: "The name for this option",
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "displayName",
          label: "The display name for this option",
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addEnum({
          name: "optionType",
          label: "The type of option",
          options(p: PropEnum) {
            p.required = true;
            p.variants = ["string", "secret"];
          },
        });
      },
    });
    c.fields.addObject({
      name: "siProperties",
      label: "SI Internal Properties",
      options(p: PropObject) {
        p.required = true;
        p.properties.addNumber({
          name: "version",
          label: "The version of this integration",
          options(p: PropNumber) {
            p.required = true;
            p.hidden = true;
            p.numberKind = "int32";
          },
        });
      },
    });
    c.addListMethod();
    c.addGetMethod();
    c.methods.addMethod({
      name: "create",
      label: "Create an Integration",
      options(p: PropMethod) {
        p.mutation = true;
        p.hidden = true;
        p.isPrivate = true;
        p.request.properties.addText({
          name: "name",
          label: "Integration Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Integration Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addLink({
          name: "options",
          label: "Options for this Integration",
          options(p: PropLink) {
            p.repeated = true;
            p.lookup = {
              typeName: "integration",
              names: ["options"],
            };
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "Si Properties",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "integration",
              names: ["siProperties"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "object",
          label: `${c.displayTypeName} Object`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "integration",
            };
          },
        });
      },
    });
  },
});
