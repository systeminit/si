import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";
import {
  attachExtraActionFunction,
  objectPropForOverride,
  propForOverride,
  suggest,
  widget,
} from "../generic/overrides.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createScalarProp,
  ExpandedPropSpec,
} from "../../spec/props.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";

// Common GCP regions as of 2025
// Based on official GCP documentation: https://cloud.google.com/about/locations
// Note: Region list retrieved from compute.googleapis.com/compute/v1/projects/{project}/regions API
const GCP_REGIONS = [
  // Africa
  { value: "africa-south1", label: "Africa South 1 (Johannesburg)" },

  // Asia Pacific
  { value: "asia-east1", label: "Asia East 1 (Taiwan)" },
  { value: "asia-east2", label: "Asia East 2 (Hong Kong)" },
  { value: "asia-northeast1", label: "Asia Northeast 1 (Tokyo)" },
  { value: "asia-northeast2", label: "Asia Northeast 2 (Osaka)" },
  { value: "asia-northeast3", label: "Asia Northeast 3 (Seoul)" },
  { value: "asia-south1", label: "Asia South 1 (Mumbai)" },
  { value: "asia-south2", label: "Asia South 2 (Delhi)" },
  { value: "asia-southeast1", label: "Asia Southeast 1 (Singapore)" },
  { value: "asia-southeast2", label: "Asia Southeast 2 (Jakarta)" },
  { value: "australia-southeast1", label: "Australia Southeast 1 (Sydney)" },
  { value: "australia-southeast2", label: "Australia Southeast 2 (Melbourne)" },

  // Europe
  { value: "europe-central2", label: "Europe Central 2 (Warsaw)" },
  { value: "europe-north1", label: "Europe North 1 (Finland)" },
  { value: "europe-north2", label: "Europe North 2 (Stockholm)" },
  { value: "europe-southwest1", label: "Europe Southwest 1 (Madrid)" },
  { value: "europe-west1", label: "Europe West 1 (Belgium)" },
  { value: "europe-west2", label: "Europe West 2 (London)" },
  { value: "europe-west3", label: "Europe West 3 (Frankfurt)" },
  { value: "europe-west4", label: "Europe West 4 (Netherlands)" },
  { value: "europe-west6", label: "Europe West 6 (Zurich)" },
  { value: "europe-west8", label: "Europe West 8 (Milan)" },
  { value: "europe-west9", label: "Europe West 9 (Paris)" },
  { value: "europe-west10", label: "Europe West 10 (Berlin)" },
  { value: "europe-west12", label: "Europe West 12 (Turin)" },

  // Middle East
  { value: "me-central1", label: "Middle East Central 1 (Doha)" },
  { value: "me-central2", label: "Middle East Central 2 (Dammam)" },
  { value: "me-west1", label: "Middle East West 1 (Tel Aviv)" },

  // North America
  { value: "northamerica-northeast1", label: "North America Northeast 1 (Montréal)" },
  { value: "northamerica-northeast2", label: "North America Northeast 2 (Toronto)" },
  { value: "northamerica-south1", label: "North America South 1 (Mexico)" },
  { value: "us-central1", label: "US Central 1 (Iowa)" },
  { value: "us-east1", label: "US East 1 (South Carolina)" },
  { value: "us-east4", label: "US East 4 (N. Virginia)" },
  { value: "us-east5", label: "US East 5 (Columbus)" },
  { value: "us-south1", label: "US South 1 (Dallas)" },
  { value: "us-west1", label: "US West 1 (Oregon)" },
  { value: "us-west2", label: "US West 2 (Los Angeles)" },
  { value: "us-west3", label: "US West 3 (Salt Lake City)" },
  { value: "us-west4", label: "US West 4 (Las Vegas)" },

  // South America
  { value: "southamerica-east1", label: "South America East 1 (São Paulo)" },
  { value: "southamerica-west1", label: "South America West 1 (Santiago)" },
];

// GCP Zones - verified from gcloud compute zones list
// Note: Some regions have non-standard zone letters (e.g., europe-west1 has b,c,d not a,b,c)
const GCP_ZONES = [
  // Africa
  { value: "africa-south1-a", label: "Africa South 1 Zone A (Johannesburg)" },
  { value: "africa-south1-b", label: "Africa South 1 Zone B (Johannesburg)" },
  { value: "africa-south1-c", label: "Africa South 1 Zone C (Johannesburg)" },

  // Asia Pacific
  { value: "asia-east1-a", label: "Asia East 1 Zone A (Taiwan)" },
  { value: "asia-east1-b", label: "Asia East 1 Zone B (Taiwan)" },
  { value: "asia-east1-c", label: "Asia East 1 Zone C (Taiwan)" },
  { value: "asia-east2-a", label: "Asia East 2 Zone A (Hong Kong)" },
  { value: "asia-east2-b", label: "Asia East 2 Zone B (Hong Kong)" },
  { value: "asia-east2-c", label: "Asia East 2 Zone C (Hong Kong)" },
  { value: "asia-northeast1-a", label: "Asia Northeast 1 Zone A (Tokyo)" },
  { value: "asia-northeast1-b", label: "Asia Northeast 1 Zone B (Tokyo)" },
  { value: "asia-northeast1-c", label: "Asia Northeast 1 Zone C (Tokyo)" },
  { value: "asia-northeast2-a", label: "Asia Northeast 2 Zone A (Osaka)" },
  { value: "asia-northeast2-b", label: "Asia Northeast 2 Zone B (Osaka)" },
  { value: "asia-northeast2-c", label: "Asia Northeast 2 Zone C (Osaka)" },
  { value: "asia-northeast3-a", label: "Asia Northeast 3 Zone A (Seoul)" },
  { value: "asia-northeast3-b", label: "Asia Northeast 3 Zone B (Seoul)" },
  { value: "asia-northeast3-c", label: "Asia Northeast 3 Zone C (Seoul)" },
  { value: "asia-south1-a", label: "Asia South 1 Zone A (Mumbai)" },
  { value: "asia-south1-b", label: "Asia South 1 Zone B (Mumbai)" },
  { value: "asia-south1-c", label: "Asia South 1 Zone C (Mumbai)" },
  { value: "asia-south2-a", label: "Asia South 2 Zone A (Delhi)" },
  { value: "asia-south2-b", label: "Asia South 2 Zone B (Delhi)" },
  { value: "asia-south2-c", label: "Asia South 2 Zone C (Delhi)" },
  { value: "asia-southeast1-a", label: "Asia Southeast 1 Zone A (Singapore)" },
  { value: "asia-southeast1-b", label: "Asia Southeast 1 Zone B (Singapore)" },
  { value: "asia-southeast1-c", label: "Asia Southeast 1 Zone C (Singapore)" },
  { value: "asia-southeast2-a", label: "Asia Southeast 2 Zone A (Jakarta)" },
  { value: "asia-southeast2-b", label: "Asia Southeast 2 Zone B (Jakarta)" },
  { value: "asia-southeast2-c", label: "Asia Southeast 2 Zone C (Jakarta)" },
  { value: "australia-southeast1-a", label: "Australia Southeast 1 Zone A (Sydney)" },
  { value: "australia-southeast1-b", label: "Australia Southeast 1 Zone B (Sydney)" },
  { value: "australia-southeast1-c", label: "Australia Southeast 1 Zone C (Sydney)" },
  { value: "australia-southeast2-a", label: "Australia Southeast 2 Zone A (Melbourne)" },
  { value: "australia-southeast2-b", label: "Australia Southeast 2 Zone B (Melbourne)" },
  { value: "australia-southeast2-c", label: "Australia Southeast 2 Zone C (Melbourne)" },

  // Europe
  { value: "europe-central2-a", label: "Europe Central 2 Zone A (Warsaw)" },
  { value: "europe-central2-b", label: "Europe Central 2 Zone B (Warsaw)" },
  { value: "europe-central2-c", label: "Europe Central 2 Zone C (Warsaw)" },
  { value: "europe-north1-a", label: "Europe North 1 Zone A (Finland)" },
  { value: "europe-north1-b", label: "Europe North 1 Zone B (Finland)" },
  { value: "europe-north1-c", label: "Europe North 1 Zone C (Finland)" },
  { value: "europe-north2-a", label: "Europe North 2 Zone A (Stockholm)" },
  { value: "europe-north2-b", label: "Europe North 2 Zone B (Stockholm)" },
  { value: "europe-north2-c", label: "Europe North 2 Zone C (Stockholm)" },
  { value: "europe-southwest1-a", label: "Europe Southwest 1 Zone A (Madrid)" },
  { value: "europe-southwest1-b", label: "Europe Southwest 1 Zone B (Madrid)" },
  { value: "europe-southwest1-c", label: "Europe Southwest 1 Zone C (Madrid)" },
  { value: "europe-west1-b", label: "Europe West 1 Zone B (Belgium)" },
  { value: "europe-west1-c", label: "Europe West 1 Zone C (Belgium)" },
  { value: "europe-west1-d", label: "Europe West 1 Zone D (Belgium)" },
  { value: "europe-west2-a", label: "Europe West 2 Zone A (London)" },
  { value: "europe-west2-b", label: "Europe West 2 Zone B (London)" },
  { value: "europe-west2-c", label: "Europe West 2 Zone C (London)" },
  { value: "europe-west3-a", label: "Europe West 3 Zone A (Frankfurt)" },
  { value: "europe-west3-b", label: "Europe West 3 Zone B (Frankfurt)" },
  { value: "europe-west3-c", label: "Europe West 3 Zone C (Frankfurt)" },
  { value: "europe-west4-a", label: "Europe West 4 Zone A (Netherlands)" },
  { value: "europe-west4-b", label: "Europe West 4 Zone B (Netherlands)" },
  { value: "europe-west4-c", label: "Europe West 4 Zone C (Netherlands)" },
  { value: "europe-west6-a", label: "Europe West 6 Zone A (Zurich)" },
  { value: "europe-west6-b", label: "Europe West 6 Zone B (Zurich)" },
  { value: "europe-west6-c", label: "Europe West 6 Zone C (Zurich)" },
  { value: "europe-west8-a", label: "Europe West 8 Zone A (Milan)" },
  { value: "europe-west8-b", label: "Europe West 8 Zone B (Milan)" },
  { value: "europe-west8-c", label: "Europe West 8 Zone C (Milan)" },
  { value: "europe-west9-a", label: "Europe West 9 Zone A (Paris)" },
  { value: "europe-west9-b", label: "Europe West 9 Zone B (Paris)" },
  { value: "europe-west9-c", label: "Europe West 9 Zone C (Paris)" },
  { value: "europe-west10-a", label: "Europe West 10 Zone A (Berlin)" },
  { value: "europe-west10-b", label: "Europe West 10 Zone B (Berlin)" },
  { value: "europe-west10-c", label: "Europe West 10 Zone C (Berlin)" },
  { value: "europe-west12-a", label: "Europe West 12 Zone A (Turin)" },
  { value: "europe-west12-b", label: "Europe West 12 Zone B (Turin)" },
  { value: "europe-west12-c", label: "Europe West 12 Zone C (Turin)" },

  // Middle East
  { value: "me-central1-a", label: "Middle East Central 1 Zone A (Doha)" },
  { value: "me-central1-b", label: "Middle East Central 1 Zone B (Doha)" },
  { value: "me-central1-c", label: "Middle East Central 1 Zone C (Doha)" },
  { value: "me-central2-a", label: "Middle East Central 2 Zone A (Dammam)" },
  { value: "me-central2-b", label: "Middle East Central 2 Zone B (Dammam)" },
  { value: "me-central2-c", label: "Middle East Central 2 Zone C (Dammam)" },
  { value: "me-west1-a", label: "Middle East West 1 Zone A (Tel Aviv)" },
  { value: "me-west1-b", label: "Middle East West 1 Zone B (Tel Aviv)" },
  { value: "me-west1-c", label: "Middle East West 1 Zone C (Tel Aviv)" },

  // North America
  { value: "northamerica-northeast1-a", label: "North America Northeast 1 Zone A (Montréal)" },
  { value: "northamerica-northeast1-b", label: "North America Northeast 1 Zone B (Montréal)" },
  { value: "northamerica-northeast1-c", label: "North America Northeast 1 Zone C (Montréal)" },
  { value: "northamerica-northeast2-a", label: "North America Northeast 2 Zone A (Toronto)" },
  { value: "northamerica-northeast2-b", label: "North America Northeast 2 Zone B (Toronto)" },
  { value: "northamerica-northeast2-c", label: "North America Northeast 2 Zone C (Toronto)" },
  { value: "northamerica-south1-a", label: "North America South 1 Zone A (Mexico)" },
  { value: "northamerica-south1-b", label: "North America South 1 Zone B (Mexico)" },
  { value: "northamerica-south1-c", label: "North America South 1 Zone C (Mexico)" },
  { value: "us-central1-a", label: "US Central 1 Zone A (Iowa)" },
  { value: "us-central1-b", label: "US Central 1 Zone B (Iowa)" },
  { value: "us-central1-c", label: "US Central 1 Zone C (Iowa)" },
  { value: "us-central1-f", label: "US Central 1 Zone F (Iowa)" },
  { value: "us-east1-b", label: "US East 1 Zone B (South Carolina)" },
  { value: "us-east1-c", label: "US East 1 Zone C (South Carolina)" },
  { value: "us-east1-d", label: "US East 1 Zone D (South Carolina)" },
  { value: "us-east4-a", label: "US East 4 Zone A (N. Virginia)" },
  { value: "us-east4-b", label: "US East 4 Zone B (N. Virginia)" },
  { value: "us-east4-c", label: "US East 4 Zone C (N. Virginia)" },
  { value: "us-east5-a", label: "US East 5 Zone A (Columbus)" },
  { value: "us-east5-b", label: "US East 5 Zone B (Columbus)" },
  { value: "us-east5-c", label: "US East 5 Zone C (Columbus)" },
  { value: "us-south1-a", label: "US South 1 Zone A (Dallas)" },
  { value: "us-south1-b", label: "US South 1 Zone B (Dallas)" },
  { value: "us-south1-c", label: "US South 1 Zone C (Dallas)" },
  { value: "us-west1-a", label: "US West 1 Zone A (Oregon)" },
  { value: "us-west1-b", label: "US West 1 Zone B (Oregon)" },
  { value: "us-west1-c", label: "US West 1 Zone C (Oregon)" },
  { value: "us-west2-a", label: "US West 2 Zone A (Los Angeles)" },
  { value: "us-west2-b", label: "US West 2 Zone B (Los Angeles)" },
  { value: "us-west2-c", label: "US West 2 Zone C (Los Angeles)" },
  { value: "us-west3-a", label: "US West 3 Zone A (Salt Lake City)" },
  { value: "us-west3-b", label: "US West 3 Zone B (Salt Lake City)" },
  { value: "us-west3-c", label: "US West 3 Zone C (Salt Lake City)" },
  { value: "us-west4-a", label: "US West 4 Zone A (Las Vegas)" },
  { value: "us-west4-b", label: "US West 4 Zone B (Las Vegas)" },
  { value: "us-west4-c", label: "US West 4 Zone C (Las Vegas)" },

  // South America
  { value: "southamerica-east1-a", label: "South America East 1 Zone A (São Paulo)" },
  { value: "southamerica-east1-b", label: "South America East 1 Zone B (São Paulo)" },
  { value: "southamerica-east1-c", label: "South America East 1 Zone C (São Paulo)" },
  { value: "southamerica-west1-a", label: "South America West 1 Zone A (Santiago)" },
  { value: "southamerica-west1-b", label: "South America West 1 Zone B (Santiago)" },
  { value: "southamerica-west1-c", label: "South America West 1 Zone C (Santiago)" },
];

// GCP asset name constants for suggestSource references
// These match the names generated by buildGcpTypeName() from the Compute Engine API
const GCP_COMPUTE = "Google Cloud Compute Engine";

export const GCP_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  ".*": {
    // Add region dropdown to "region" prop on all resources
    region: widget("ComboBox", GCP_REGIONS),
    // Add zone dropdown to "zone" prop on all resources
    zone: widget("ComboBox", GCP_ZONES),

    // Network references - suggest from Networks asset selfLink
    network: suggest(`${GCP_COMPUTE} Networks`, "selfLink"),

    // Subnetwork references - suggest from Subnetworks asset selfLink
    subnetwork: suggest(`${GCP_COMPUTE} Subnetworks`, "selfLink"),

    // Instance references - suggest from Instances asset selfLink
    instance: suggest(`${GCP_COMPUTE} Instances`, "selfLink"),

    // Disk references - suggest from Disks asset selfLink
    // Note: "source" is often used for disk references in attachedDisks
    source: suggest(`${GCP_COMPUTE} Disks`, "selfLink"),

    // Image references - suggest from Images asset selfLink
    sourceImage: suggest(`${GCP_COMPUTE} Images`, "selfLink"),

    // Health check references - suggest from HealthChecks asset selfLink
    healthCheck: suggest(`${GCP_COMPUTE} HealthChecks`, "selfLink"),

    // Backend service references - suggest from BackendServices asset selfLink
    backendService: suggest(`${GCP_COMPUTE} BackendServices`, "selfLink"),

    // Instance group references - suggest from InstanceGroups asset selfLink
    instanceGroup: suggest(`${GCP_COMPUTE} InstanceGroups`, "selfLink"),

    // Instance template references - suggest from InstanceTemplates asset selfLink
    instanceTemplate: suggest(`${GCP_COMPUTE} InstanceTemplates`, "selfLink"),

    // Target pool references - suggest from TargetPools asset selfLink
    targetPool: suggest(`${GCP_COMPUTE} TargetPools`, "selfLink"),

    // Firewall references - suggest from Firewalls asset selfLink
    firewall: suggest(`${GCP_COMPUTE} Firewalls`, "selfLink"),

    // Address references - suggest from Addresses asset selfLink
    address: suggest(`${GCP_COMPUTE} Addresses`, "selfLink"),

    // Forwarding rule references - suggest from ForwardingRules asset selfLink
    forwardingRule: suggest(`${GCP_COMPUTE} ForwardingRules`, "selfLink"),

    // URL map references - suggest from UrlMaps asset selfLink
    urlMap: suggest(`${GCP_COMPUTE} UrlMaps`, "selfLink"),

    // Target HTTP proxy references
    targetHttpProxy: suggest(`${GCP_COMPUTE} TargetHttpProxies`, "selfLink"),

    // Target HTTPS proxy references
    targetHttpsProxy: suggest(`${GCP_COMPUTE} TargetHttpsProxies`, "selfLink"),

    // SSL certificate references
    sslCertificate: suggest(`${GCP_COMPUTE} SslCertificates`, "selfLink"),
  },
};

// GKE asset name constant
const GCP_GKE = "Google Cloud Kubernetes Engine";

export const GCP_SCHEMA_OVERRIDES: Map<
  string,
  SchemaOverrideFn | SchemaOverrideFn[]
> = new Map([
  // GKE Clusters - add location prop for URL construction and exclude deprecated fields
  [
    `${GCP_GKE} Projects.Locations.Clusters`,
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const domainPath = variant.domain.metadata.propPath;

      // Add a writable "location" prop with ComboBox for regions
      // This is used by the action functions to auto-construct the `parent` URL parameter
      const locationProp = createScalarProp("location", "string", domainPath, true);
      locationProp.data.widgetKind = "ComboBox";
      locationProp.data.widgetOptions = GCP_REGIONS.map(r => ({ label: r.label, value: r.value }));
      locationProp.data.documentation = "The region or zone where the cluster will be created (e.g., us-central1 for regional, us-central1-a for zonal).";
      // Mark as required - needed to construct the parent URL parameter
      locationProp.data.validationFormat = JSON.stringify({ type: "string", flags: { presence: "required" } });
      variant.domain.entries.push(locationProp);

      // Mark "location" as excluded from codegen (not sent in request body, only used for URL)
      // Also mark deprecated loggingService/monitoringService as excluded (conflict with loggingConfig/monitoringConfig)
      const extraProp = objectPropForOverride(variant.domain, "extra");
      const propUsageMapProp = propForOverride(extraProp, "PropUsageMap");
      if (propUsageMapProp.data?.defaultValue) {
        const propUsageMap = JSON.parse(
          propUsageMapProp.data.defaultValue as string,
        );
        propUsageMap.excluded = propUsageMap.excluded || [];
        propUsageMap.excluded.push("location"); // Only for URL, not request body
        propUsageMap.excluded.push("loggingService"); // Deprecated, conflicts with loggingConfig
        propUsageMap.excluded.push("monitoringService"); // Deprecated, conflicts with monitoringConfig
        propUsageMapProp.data.defaultValue = JSON.stringify(propUsageMap);
      }
    },
  ],
  [
    "Google Cloud Compute Engine InstanceGroups",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const domainPath = variant.domain.metadata.propPath;

      // Create "instances" array prop for add/remove instances actions
      const itemProp = createScalarProp("instance", "string", [...domainPath, "instances"], false);
      addPropSuggestSource(itemProp, { schema: `${GCP_COMPUTE} Instances`, prop: "/resource_value/selfLink" });

      const instancesProp: ExpandedPropSpec = {
        kind: "array",
        name: "instances",
        uniqueId: ulid(),
        cfProp: undefined,
        typeProp: itemProp,
        data: {
          name: "instances",
          validationFormat: null,
          defaultValue: null,
          funcUniqueId: null,
          inputs: [],
          widgetKind: "Array",
          widgetOptions: null,
          hidden: false,
          docLink: null,
          documentation: "Instance selfLinks or names to add/remove from this instance group.",
          uiOptionals: null,
        },
        metadata: {
          createOnly: false,
          readOnly: false,
          writeOnly: false,
          primaryIdentifier: false,
          propPath: [...domainPath, "instances"],
          required: false,
        },
      };

      variant.domain.entries.push(instancesProp);

      // Mark "instances" as excluded from codegen (not sent to create/update API)
      const extraProp = objectPropForOverride(variant.domain, "extra");
      const propUsageMapProp = propForOverride(extraProp, "PropUsageMap");
      if (propUsageMapProp.data?.defaultValue) {
        const propUsageMap = JSON.parse(
          propUsageMapProp.data.defaultValue as string,
        );
        // Add "instances" to the list of properties to exclude from API calls
        propUsageMap.excluded = propUsageMap.excluded || [];
        propUsageMap.excluded.push("instances");
        propUsageMapProp.data.defaultValue = JSON.stringify(propUsageMap);
      }

      // Add "Add Instances" action
      const {
        func: addInstancesFunc,
        actionFuncSpec: addInstancesActionFuncSpec,
      } = attachExtraActionFunction(
        "./src/pipelines/gcp/funcs/overrides/InstanceGroups/actions/addInstances.ts",
        "Add Instances to Group",
        "other",
        "7d08100a4de7e37dc1bc55fd82ca71ff4938718436d93b8bf38f775d1d65d941",
      );
      spec.funcs.push(addInstancesFunc);
      variant.actionFuncs.push(addInstancesActionFuncSpec);

      // Add "Remove Instances" action
      const {
        func: removeInstancesFunc,
        actionFuncSpec: removeInstancesActionFuncSpec,
      } = attachExtraActionFunction(
        "./src/pipelines/gcp/funcs/overrides/InstanceGroups/actions/removeInstances.ts",
        "Remove Instances from Group",
        "other",
        "1729e7ecf77967b8a440f7a25057d4dac5c0e0a8fc1831c42359ead6f7f7fd7d",
      );
      spec.funcs.push(removeInstancesFunc);
      variant.actionFuncs.push(removeInstancesActionFuncSpec);
    },
  ],
]);
