import {
  PropObject,
  PropMethod,
  PropLink,
  PropText,
  PropNumber,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "integrationInstance",
  displayTypeName: "An instance of an integration with another system",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "integrationId"],
      typeName: "integration",
    });
    c.fields.addObject({
      name: "optionValues",
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
          name: "value",
          label: "The value for this option",
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addLink({
          name: "optionType",
          label: "The type of option",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "integration",
              names: ["options", "optionType"],
            };
          },
        });
      },
    });
    /// TODO: COMPLETE THE PATTERN AROUND siProperties and deeper nested objects - need to pass it
    //  or define it from some external source.
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
          name: "integrationId",
          label: "Integration ID",
          options(p) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "enabledWorkspaceIdList",
          label:
            "List of workspace id's this integration instance is enabled on",
          options(p: PropText) {
            p.repeated = true;
            p.required = true;
          },
        });
        p.properties.addText({
          name: "enabledOrganizationIdList",
          label:
            "List of organization id's this integration instance is enabled on",
          options(p: PropText) {
            p.repeated = true;
            p.required = true;
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
          name: "optionValues",
          label: "Options for this Integration Instance",
          options(p: PropLink) {
            p.required = true;
            p.repeated = true;
            p.lookup = {
              typeName: "integrationInstance",
              names: ["optionValues"],
            };
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "Si Properties",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "integrationInstance",
              names: ["siProperties"],
            };
          },
        });

        p.reply.properties.addLink({
          name: "object",
          label: `${c.displayTypeName} Object`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "integrationInstance",
            };
          },
        });
      },
    });
  },
});
