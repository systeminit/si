import {
  PropEnum,
  PropObject,
  PropText,
  PropLink,
  PropNumber,
  Component,
} from "@/components/prelude";

import { registry } from "@/componentRegistry";

registry.component({
  typeName: "data",
  displayTypeName: "SI Data",
  noStd: true,
  options(c: Component) {
    c.internalOnly.addObject({
      name: "storable",
      label: "SI Internal Storable Data",
      options(p: PropObject) {
        p.universal = true;
        p.properties.addText({
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
        p.properties.addText({
          name: "naturalKey",
          label: "Natural Key",
          options(p) {
            p.readOnly = true;
            p.hidden = true;
            p.required = true;
            p.universal = true;
          },
        });
        p.properties.addText({
          name: "typeName",
          label: "Type Name",
          options(p) {
            p.readOnly = true;
            p.hidden = true;
            p.required = true;
            p.universal = true;
          },
        });
        p.properties.addText({
          name: "displayTypeName",
          label: "Display Type Name",
          options(p) {
            p.readOnly = true;
            p.required = true;
            p.universal = true;
          },
        });
      },
    });
    c.internalOnly.addObject({
      name: "pageToken",
      label: "Page Token",
      options(p: PropObject) {
        p.universal = true;
        p.properties.addLink({
          name: "query",
          label: "Query",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              component: "data",
              propType: "internalOnly",
              names: ["query"],
            };
          },
        });
        p.properties.addNumber({
          name: "pageSize",
          label: "Page Size",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "int32";
          },
        });
        p.properties.addText({
          name: "orderBy",
          label: "Order by",
          options(p) {
            p.universal = true;
          },
        });
        p.properties.addEnum({
          name: "orderByDirection",
          label: "Order by direction",
          options(p: PropEnum) {
            p.universal = true;
            p.variants = ["asc", "desc"];
          },
        });
        p.properties.addText({
          name: "itemId",
          label: "Item ID",
          options(p) {
            p.universal = true;
          },
        });
        p.properties.addText({
          name: "containedWithin",
          label: "Contained Within",
          options(p) {
            p.universal = true;
          },
        });
      },
    });
    c.internalOnly.addObject({
      name: "query",
      label: "Query",
      options(p: PropObject) {
        p.properties.addEnum({
          name: "booleanTerm",
          label: "Query Boolean Logic",
          options(p: PropEnum) {
            p.variants = ["and", "or"];
            p.universal = true;
          },
        });
        p.properties.addBool({
          name: "isNot",
          label: "Is Not",
          options(p) {
            p.universal = true;
          },
        });
        p.properties.addObject({
          name: "expressionOption",
          label: "Expression Option",
          options(p: PropObject) {
            p.repeated = true;
            p.universal = true;
            p.properties.addLink({
              name: "query",
              label: "Query",
              options(p: PropLink) {
                p.lookup = {
                  component: "data",
                  propType: "internalOnly",
                  names: ["query"],
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
  },
});

