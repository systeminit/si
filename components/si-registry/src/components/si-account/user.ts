import { PropObject, PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";

registry.system({
  typeName: "user",
  displayTypeName: "A System Initiative User",
  siPathName: "si-account",
  serviceName: "account",
  options(c) {
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
      },
    });
    c.fields.addObject({
      name: "siProperties",
      label: "SI Internal Properties",
      options(p: PropObject) {
        p.properties.addText({
          name: "billingAccountId",
          label: "Billing Account ID",
          options(p) {
            p.required = true;
          },
        });
      },
    });
    // Create
    c.methods.addMethod({
      name: "create",
      label: "Create a User",
      options(p: PropMethod) {
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
        p.request.properties.addText({
          name: "billingAccountId",
          label: "The billing account for this user",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "user",
          label: "The User",
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
