import {
  PropObject,
  PropNumber,
  PropMethod,
  PropLink,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.base({
  typeName: "item",
  displayTypeName: "An item",
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

    c.fields.addText({
      name: "id",
      label: `${c.displayTypeName} ID`,
      options(p) {
        p.universal = true;
        p.readOnly = true;
        p.required = true;
      },
    });
    c.fields.addText({
      name: "name",
      label: `${c.displayTypeName} Name`,
      options(p) {
        p.universal = true;
        p.readOnly = true;
        p.required = true;
      },
    });
    c.fields.addText({
      name: "displayName",
      label: `${c.displayTypeName} Display Name`,
      options(p) {
        p.universal = true;
        p.readOnly = true;
        p.required = true;
      },
    });
    c.fields.addLink({
      name: "siStorable",
      label: "SI Storable",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          typeName: "dataStorable",
        };
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

    c.methods.addMethod({
      name: "get",
      label: `Get an Item`,
      options(p: PropMethod) {
        p.request.properties.addText({
          name: "id",
          label: `Item ID`,
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `The Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: c.typeName,
            };
          },
        });
      },
    });

    c.methods.addMethod({
      name: "list",
      label: `List Items`,
      options(p: PropMethod) {
        p.universal = true;
        p.request.properties.addLink({
          name: "query",
          label: "Query",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              typeName: "dataQuery",
            };
          },
        });
        p.request.properties.addNumber({
          name: "pageSize",
          label: "Page Size",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "uint32";
          },
        });
        p.request.properties.addText({
          name: "orderBy",
          label: "Order By",
          options(p) {
            p.universal = true;
          },
        });
        p.request.properties.addLink({
          name: "orderByDirection",
          label: "Order By Direction",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              typeName: "dataPageToken",
              names: ["orderByDirection"],
            };
          },
        });
        p.request.properties.addText({
          name: "pageToken",
          label: "Page Token",
          options(p) {
            p.universal = true;
          },
        });
        p.request.properties.addText({
          name: "scopeByTenantId",
          label: "Scope By Tenant ID",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.properties.addLink({
          name: "items",
          label: "Items",
          options(p: PropLink) {
            p.universal = true;
            p.required = true;
            p.repeated = true;
            p.lookup = {
              typeName: c.typeName,
            };
          },
        });
        p.reply.properties.addNumber({
          name: "totalCount",
          label: "Total Count",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "uint32";
          },
        });
        p.reply.properties.addText({
          name: "nextPageToken",
          label: "Next Page Token",
          options(p) {
            p.universal = true;
          },
        });
      },
    });
  },
});
