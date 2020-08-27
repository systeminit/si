import { PropSelect, PropText, PropObject } from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "resource",
  displayTypeName: "A System Initiative Resource",
  siPathName: "si-core",
  serviceName: "core",
  options(c: SystemObject) {
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
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "workspaceId",
          label: "Organization ID",
          options(p: PropText) {
            p.required = true;
          },
        });
      },
    });

    c.fields.addText({
      name: "entityId",
      label: "entityId",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.fields.addText({
      name: "nodeId",
      label: "nodeId",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.fields.addText({
      name: "kind",
      label: "kind",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.fields.addSelect({
      name: "status",
      label: "status",
      options(p: PropSelect) {
        p.required = true;
        p.baseValidation = p.baseValidation.allow(
          "PENDING",
          "CREATED",
          "FAILED",
          "DELETED",
        );
        p.options = [
          { key: "pending", value: "PENDING" }, // There is an action pending for this resource
          { key: "created", value: "CREATED" }, // The resource exists, but either has no status endpoint or it hasn't reported yet
          { key: "failed", value: "FAILED" }, // The resource creation failed
          { key: "deleted", value: "DELETED" }, // The resource has been removed
        ];
      },
    });

    c.fields.addSelect({
      name: "health",
      label: "health",
      options(p: PropSelect) {
        p.required = true;
        p.baseValidation = p.baseValidation.allow(
          "OK",
          "WARNING",
          "ERROR",
          "UNKNOWN",
        );
        p.options = [
          { key: "ok", value: "OK" }, // There is an action pending for this resource
          { key: "warning", value: "WARNING" }, // The resource exists, but either has no status endpoint or it hasn't reported yet
          { key: "error", value: "ERROR" }, // The resource creation failed
          { key: "unknown", value: "UNKNOWN" }, // The resource has been removed
        ];
      },
    });

    c.fields.addText({
      name: "data",
      label: "data",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.addListMethod();
    c.addGetMethod();
    c.addCreateMethod();
    c.addUpdateMethod();
  },
});
