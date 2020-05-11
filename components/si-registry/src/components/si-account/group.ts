import {
  PropObject,
  PropMethod,
  PropLink,
  PropText,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "group",
  displayTypeName: "A System Initiative User Group",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    // The magical add list association lives here! Yay! Progress
    c.fields.addText({
      name: "userIds",
      label: "User IDs of our groups members",
      options(p: PropText) {
        p.repeated = true;
        p.required = true;
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
        p.repeated = true;
        p.lookup = {
          typeName: "capability",
        };
      },
    });

    c.addListMethod();
    c.addGetMethod();
    c.methods.addMethod({
      name: "create",
      label: "Create a Group",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "name",
          label: "Group Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Group Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "userIds",
          label: "Group user IDs",
          options(p) {
            p.repeated = true;
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "The SI Properties for this User",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "group",
              names: ["siProperties"],
            };
          },
        });
        p.request.properties.addLink({
          name: "capabilities",
          label: "Authorized capabilities for this user",
          options(p: PropLink) {
            p.hidden = true;
            p.repeated = true;
            p.lookup = {
              typeName: "capability",
            };
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${c.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "group",
            };
          },
        });
      },
    });
  },
});
