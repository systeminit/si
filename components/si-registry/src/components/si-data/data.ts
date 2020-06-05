import {
  PropEnum,
  PropObject,
  PropText,
  PropLink,
  PropNumber,
  PropBool,
} from "../../components/prelude";

import { registry } from "../../registry";

registry.base({
  typeName: "dataStorable",
  displayTypeName: "SI Internal Storable Data",
  serviceName: "data",
  options(c) {
    c.fields.addText({
      name: "tenantIds",
      label: "Tenant IDs",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.repeated = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "naturalKey",
      label: "Natural Key",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "typeName",
      label: "Type Name",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "viewContext",
      label: "View context tags",
      options(p) {
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "changeSetId",
      label: "The Change Set ID for this item",
      options(p) {
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "itemId",
      label: "The canonical ID for this item",
      options(p) {
        p.universal = true;
      },
    });
    c.fields.addNumber({
      name: "changeSetEntryCount",
      label: "Order for the Change Set Entry",
      options(p: PropNumber) {
        p.numberKind = "uint64";
        p.universal = true;
      },
    });
    c.fields.addEnum({
      name: "changeSetEventType",
      label: "The Change Set event type",
      options(p: PropEnum) {
        p.universal = true;
        p.variants = ["create", "update", "delete", "action"];
      },
    });
    c.fields.addBool({
      name: "changeSetExecuted",
      label: "has this been executed",
      options(p: PropBool) {
        p.universal = true;
        p.baseDefaultValue = false;
      },
    });
    c.fields.addBool({
      name: "deleted",
      label: "has this been deleted",
      options(p: PropBool) {
        p.universal = true;
        p.baseDefaultValue = false;
      },
    });
  },
});

registry.base({
  typeName: "dataPageToken",
  displayTypeName: "Page Token",
  serviceName: "data",
  options(c) {
    c.fields.addLink({
      name: "query",
      label: "Query",
      options(p: PropLink) {
        p.universal = true;
        p.lookup = {
          typeName: "dataQuery",
        };
      },
    });
    c.fields.addNumber({
      name: "pageSize",
      label: "Page Size",
      options(p: PropNumber) {
        p.universal = true;
        p.numberKind = "uint32";
      },
    });
    c.fields.addText({
      name: "orderBy",
      label: "Order by",
      options(p) {
        p.universal = true;
      },
    });
    c.fields.addEnum({
      name: "orderByDirection",
      label: "Order by direction",
      options(p: PropEnum) {
        p.universal = true;
        p.variants = ["asc", "desc"];
      },
    });
    c.fields.addText({
      name: "itemId",
      label: "Item ID",
      options(p) {
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "containedWithin",
      label: "Contained Within",
      options(p) {
        p.universal = true;
      },
    });
  },
});

registry.base({
  typeName: "dataQuery",
  displayTypeName: "Query",
  serviceName: "data",
  options(c) {
    c.fields.addEnum({
      name: "booleanTerm",
      label: "Query Boolean Logic",
      options(p: PropEnum) {
        p.variants = ["and", "or"];
        p.universal = true;
      },
    });
    c.fields.addBool({
      name: "isNot",
      label: "Is Not",
      options(p) {
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "viewContext",
      label: "Filter by View Context Tag",
      options(p) {
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "changeSetId",
      label: "Filter by Change Set ID",
      options(p) {
        p.universal = true;
      },
    });
    c.fields.addObject({
      name: "items",
      label: "Expression Option",
      options(p: PropObject) {
        p.repeated = true;
        p.universal = true;
        p.properties.addLink({
          name: "query",
          label: "Query",
          options(p: PropLink) {
            p.lookup = {
              typeName: "dataQuery",
            };
          },
        });
        p.properties.addObject({
          name: "expression",
          label: "Query Expression",
          options(p: PropObject) {
            p.universal = true;
            p.properties.addText({
              name: "field",
              label: "Field",
              options(p: PropText) {
                p.universal = true;
                p.required = true;
              },
            });
            p.properties.addText({
              name: "value",
              label: "Value",
              options(p: PropText) {
                p.universal = true;
                p.required = true;
              },
            });
            p.properties.addEnum({
              name: "comparison",
              label: "Query Comparison",
              options(p: PropEnum) {
                p.universal = true;
                p.variants = [
                  "equals",
                  "notEquals",
                  "contains",
                  "like",
                  "notLike",
                ];
              },
            });
            p.properties.addEnum({
              name: "fieldType",
              label: "Query Field Type",
              options(p: PropEnum) {
                p.universal = true;
                p.variants = ["string", "int"];
              },
            });
          },
        });
      },
    });
  },
});
