import { widget, suggest, addScalarProp } from "../generic/overrides.ts";
import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";

const HETZNER_LOCATIONS = ["fsn1", "nbg1", "hel1", "ash", "hil", "sin"];

// Hetzner-specific property overrides (empty for now)
export const HETZNER_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  ".*": {
    // Add location dropdown and suggestion to "location" prop on all resources
    location: [
      suggest("Hetzner::Cloud::Locations", "/domain/name"),
      widget("ComboBox", HETZNER_LOCATIONS),
    ],
  },
  "Hetzner::Cloud::Locations": {
    name: widget("ComboBox", HETZNER_LOCATIONS),
  },
  "Hetzner::Cloud::Servers": {
    "ssh_keys/ssh_keysItem": suggest("Hetzner::Cloud::SshKeys", "/domain/name"),
  },
};

// Hetzner-specific schema overrides!!!
export const HETZNER_SCHEMA_OVERRIDES = new Map<string, SchemaOverrideFn>([
  //
  // Add Hetzner::Cloud::Locations.name so it can be selected and filled in
  //
  ["Hetzner::Cloud::Locations", addScalarProp("/domain/name", "string", true)],
]);
