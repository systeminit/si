import {
  PropObject,
  PropMethod,
  PropLink,
  PropNumber,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "integrationService",
  displayTypeName: "An service within an integration",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.migrateable = true;

    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "integrationId"],
      typeName: "integration",
    });
    c.fields.addObject({
      name: "siProperties",
      label: "SI Internal Properties",
      options(p: PropObject) {
        p.required = true;
        p.properties.addText({
          name: "integrationId",
          label: "Integration ID",
          options(p) {
            p.required = true;
          },
        });
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

    c.addListMethod({ isPrivate: true });
    c.addGetMethod();
    c.methods.addMethod({
      name: "create",
      label: "Create an Integration Servcie",
      options(p: PropMethod) {
        p.mutation = true;
        p.hidden = true;
        p.isPrivate = true;
        p.request.properties.addText({
          name: "name",
          label: "Integration Service Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Integration Service Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "Si Properties",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "integrationService",
              names: ["siProperties"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${c.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "integrationService",
            };
          },
        });
      },
    });
  },
});
