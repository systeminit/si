import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { GcpDiscoveryDocument } from "./schema.ts";
import { JSONSchema } from "../draft_07.ts";

export function parseGcpDiscoveryDocument(
  _doc: GcpDiscoveryDocument,
): ExpandedPkgSpec[] {
  return [];
}

export function normalizeGcpProperty(prop: JSONSchema): JSONSchema {
  return prop;
}

export function extractMethodsFromResource(_resource: any): {
  get?: any;
  list?: any;
  insert?: any;
  update?: any;
  patch?: any;
  delete?: any;
} {
  return {};
}
