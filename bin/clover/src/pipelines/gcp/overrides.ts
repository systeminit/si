import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";
import { widget } from "../generic/overrides.ts";

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

export const GCP_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  ".*": {
    // Add region dropdown to "region" prop on all resources
    region: widget("ComboBox", GCP_REGIONS),
  },
};

export const GCP_SCHEMA_OVERRIDES: Map<
  string,
  SchemaOverrideFn | SchemaOverrideFn[]
> = new Map();
