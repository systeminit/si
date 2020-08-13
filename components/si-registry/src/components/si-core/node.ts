import {
  PropEnum,
  PropText,
  PropNumber,
  PropObject,
  PropMethod,
  PropLink,
} from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

// Nodes have a list of sockets
// An edge links a head/tail to a node+socket

registry.system({
  typeName: "node",
  displayTypeName: "A System Initiative Node",
  siPathName: "si-core",
  serviceName: "core",
  options(c: SystemObject) {
    c.naturalKey = "entityId";
    c.fields.addText({
      name: "entityId",
      label: "Entity ID",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.fields.addObject({
      name: "sockets",
      label: "Node Sockets",
      options(p: PropObject) {
        p.required = true;
        p.repeated = true;
        p.properties.addText({
          name: "name",
          label: "Socket name",
          options(p: PropText) {
            p.required = true;
          },
        });
      },
    });
    c.fields.addObject({
      name: "position",
      label: "Node Position",
      options(p: PropObject) {
        p.required = true;
        p.properties.addNumber({
          name: "x",
          label: "X position",
          options(p: PropNumber) {
            p.required = true;
            p.numberKind = "int32";
          },
        });
        p.properties.addNumber({
          name: "y",
          label: "Y position",
          options(p: PropNumber) {
            p.required = true;
            p.numberKind = "int32";
          },
        });
      },
    });
    c.fields.addEnum({
      name: "nodeKind",
      label: "The kind of node this is",
      options(p: PropEnum) {
        p.variants = ["entity"];
        p.baseDefaultValue = "entity";
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
    c.addListMethod();
    c.addGetMethod();
    c.addCreateMethod();

    c.methods.addMethod({
      name: "setPosition",
      label: "Set a nodes position",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "id",
          label: "Node ID",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addLink({
          name: "position",
          label: "Node Position",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "node",
              names: ["position"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: "Updated item",
          options(p: PropLink) {
            p.lookup = {
              typeName: "node",
            };
          },
        });
      },
    });
  },
});
