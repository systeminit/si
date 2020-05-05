import { PropObject, PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "billingAccount",
  displayTypeName: "System Initiative Billing Account",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.associations.hasMany({
      fieldName: "users",
      typeName: "user",
    });
    c.associations.hasMany({
      fieldName: "organizations",
      typeName: "organization",
    });
    c.associations.hasMany({
      fieldName: "integrationInstances",
      typeName: "integrationInstance",
    });

    c.addListMethod();
    c.addGetMethod();

    c.methods.addMethod({
      name: "signup",
      label: "Create a Billing Account and Administrative User",
      options(p: PropMethod) {
        p.mutation = true;
        p.skipAuth = true;
        p.request.properties.addObject({
          name: "billingAccount",
          label: "Billing Account Information",
          options(p: PropObject) {
            p.required = true;
            p.properties.addText({
              name: "name",
              label: "Billing Account Name",
              options(p) {
                p.required = true;
              },
            });
            p.properties.addText({
              name: "displayName",
              label: "Billing Account Display Name",
              options(p) {
                p.required = true;
              },
            });
          },
        });
        p.request.properties.addObject({
          name: "user",
          label: "User Information",
          options(p: PropObject) {
            p.required = true;
            p.properties.addText({
              name: "name",
              label: "User Name",
              options(p) {
                p.required = true;
              },
            });
            p.properties.addText({
              name: "displayName",
              label: "User Display Name",
              options(p) {
                p.required = true;
              },
            });
            p.properties.addText({
              name: "email",
              label: "A valid email address",
              options(p) {
                p.universal = true;
                p.required = true;
              },
            });
            p.properties.addPassword({
              name: "password",
              label: "The users password hash",
              options(p) {
                p.universal = true;
                p.required = true;
                p.hidden = true;
              },
            });
          },
        });
        p.reply.properties.addLink({
          name: "billingAccount",
          label: `Billing Account Object`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "billingAccount",
            };
          },
        });
        p.reply.properties.addLink({
          name: "user",
          label: `User Object`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "user",
            };
          },
        });
      },
    });

    c.methods.addMethod({
      name: "create",
      label: "Create a Billing Account",
      options(p: PropMethod) {
        p.mutation = true;
        p.isPrivate = true;
        p.request.properties.addText({
          name: "name",
          label: "Billing Account Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Billing Account Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${this.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "billingAccount",
            };
          },
        });
      },
    });
  },
});
