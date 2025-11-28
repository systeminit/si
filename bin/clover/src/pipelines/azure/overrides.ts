import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import {
  attachQualificationFunction,
  fixNames,
  objectPropForOverride,
  propForOverride,
} from "../generic/overrides.ts";

export const AZURE_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {};

export const AZURE_SCHEMA_OVERRIDES = new Map<string, SchemaOverrideFn>([
  [
    "Microsoft.Web/serverfarms",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const extendedLocationProp = objectPropForOverride(
        variant.domain,
        "extendedLocation",
      );

      const nameProp = propForOverride(extendedLocationProp, "name");
      nameProp.joiValidation = undefined;
      nameProp.data.validationFormat = "{}";
      nameProp.metadata.required = false;
    },
  ],
  [
    "Microsoft.Web/sites",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      
      const domainId = variant.domain.uniqueId;
      if (!domainId) return;

      const { func: validationLinuxFxVersionFunc, leafFuncSpec: leafFuncSpecDetails } =
        attachQualificationFunction(
          "./src/pipelines/azure/funcs/overrides/Microsoft.Web.sites/qualifications/validateLinuxFxVersion.ts",
          "Validate Linux Fx Version",
          "c9b5cd9e6c498e4b6c7f40fc3a6f94b17d6d88b188c6e3f21c52b185f6fa9a5d",
          domainId,
        );
      spec.funcs.push(validationLinuxFxVersionFunc);
      variant.leafFunctions.push(leafFuncSpecDetails);

      const extendedLocationProp = objectPropForOverride(
        variant.domain,
        "extendedLocation",
      );
      if (!extendedLocationProp) {
        throw new Error("Can't find extendedLocation prop");
      }

      const nameProp = propForOverride(extendedLocationProp, "name");
      if (!nameProp) {
        throw new Error("Can't find name prop");
      }
      nameProp.joiValidation = undefined;
      nameProp.data.validationFormat = "{}";
      nameProp.metadata.required = false;
    },
  ],
  [
    "Microsoft.Aad/domainServices/ouContainer",
    fixNames({
      categoryName: "Microsoft.AAD",
      schemaName: "Microsoft.AAD/domainServices/ouContainer",
    }),
  ],
  [
    "microsoft.insights/guestDiagnosticSettings",
    fixNames({
      categoryName: "Microsoft.Insights",
      schemaName: "Microsoft.Insights/guestDiagnosticSettings",
    }),
  ],
  [
    "microsoft.insights/components/linkedStorageAccounts",
    fixNames({
      categoryName: "Microsoft.Insights",
      schemaName: "Microsoft.Insights/components/linkedStorageAccounts",
    }),
  ],
  [
    "microsoft.alertsManagement/smartDetectorAlertRules",
    fixNames({
      categoryName: "Microsoft.AlertsManagement",
      schemaName: "Microsoft.AlertsManagement/smartDetectorAlertRules",
    }),
  ],
  [
    "Microsoft.DBForMySql/flexibleServers/keys",
    fixNames({
      categoryName: "Microsoft.DBforMySQL",
      schemaName: "Microsoft.DBforMySQL/flexibleServers/keys",
    }),
  ],
  [
    "Microsoft.DBForPostgreSql/flexibleServers/keys",
    fixNames({
      categoryName: "Microsoft.DBforPostgreSQL",
      schemaName: "Microsoft.DBforPostgreSQL/flexibleServers/keys",
    }),
  ],
]);
