import {
  PropObject,
  PropMethod,
  PropLink,
  PropNumber,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "user",
  displayTypeName: "A System Initiative User",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.naturalKey = "email";

    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });

    c.fields.addText({
      name: "email",
      label: "A valid email address",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
    c.fields.addPassword({
      name: "password",
      label: "The users password hash",
      options(p) {
        p.universal = true;
        p.required = true;
        p.hidden = true;
        p.skip = true;
      },
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
      },
    });
    c.fields.addLink({
      name: "capabilities",
      label: "Authorized capabilities for this user",
      options(p: PropLink) {
        p.hidden = true;
        p.lookup = {
          typeName: "capability",
        };
      },
    });

    c.addListMethod();
    c.addGetMethod();

    c.methods.addMethod({
      name: "loginInternal",
      label: "Login",
      options(p: PropMethod) {
        p.isPrivate = true;
        p.skipAuth = true;
        p.request.properties.addText({
          name: "email",
          label: "Email",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addPassword({
          name: "password",
          label: "Password",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "billingAccountName",
          label: "Billing Account",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addBool({
          name: "authenticated",
          label: "Authenticated",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addText({
          name: "userId",
          label: "User Id",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addText({
          name: "billingAccountId",
          label: "Billing Account Id",
          options(p) {
            p.required = true;
          },
        });
      },
    });

    c.methods.addMethod({
      name: "create",
      label: "Create a User",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "name",
          label: "User Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "User Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "email",
          label: "Users email address",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addPassword({
          name: "password",
          label: "Users password",
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
              typeName: "user",
              names: ["siProperties"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${c.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "user",
            };
          },
        });
      },
    });
  },
});
