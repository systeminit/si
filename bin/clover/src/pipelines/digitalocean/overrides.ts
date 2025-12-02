import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";
import { widget } from "../generic/overrides.ts";

const DIGITALOCEAN_REGIONS = [
  { value: "nyc1", label: "New York 1" },
  { value: "sfo1", label: "San Francisco 1" },
  { value: "nyc2", label: "New York 2" },
  { value: "ams2", label: "Amsterdam 2" },
  { value: "sgp1", label: "Singapore 1" },
  { value: "lon1", label: "London 1" },
  { value: "nyc3", label: "New York 3" },
  { value: "ams3", label: "Amsterdam 3" },
  { value: "fra1", label: "Frankfurt 1" },
  { value: "tor1", label: "Toronto 1" },
  { value: "sfo2", label: "San Francisco 2" },
  { value: "blr1", label: "Bangalore 1" },
  { value: "sfo3", label: "San Francisco 3" },
  { value: "syd1", label: "Sydney 1" },
  { value: "atl1", label: "Atlanta 1" },
]

// Property-level overrides for specific DigitalOcean resources
export const DIGITALOCEAN_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  ".*": {
    // Add region dropdown to "region" prop on all resources
    region: widget("ComboBox", DIGITALOCEAN_REGIONS),
  },
  // Add property overrides here as needed
  // Example:
  // "DigitalOcean/droplets": {
  //   "name": widget("text"),
  // },
};

// Schema-level overrides for specific DigitalOcean resources
export const DIGITALOCEAN_SCHEMA_OVERRIDES = new Map<
  string,
  SchemaOverrideFn | SchemaOverrideFn[]
>(
  Object.entries({
    // Add schema overrides here as needed
    // Example:
    // "DigitalOcean/droplets": [
    //   addScalarProp("/domain/region", "string", true),
    // ],
  }),
);
