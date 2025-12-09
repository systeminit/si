import { FetchSchemaOptions } from "../types.ts";
import { GcpDiscoveryDocument } from "./schema.ts";

export async function fetchGcpDiscoveryDocuments(
  _options: FetchSchemaOptions,
): Promise<void> {
  throw new Error("Not implemented");
}

export async function loadGcpServicesIndex(): Promise<{
  services: Array<{ name: string; version: string; discoveryRestUrl: string }>;
}> {
  throw new Error("Not implemented");
}

export async function loadGcpDiscoveryDocument(
  _service: string,
  _version: string,
): Promise<GcpDiscoveryDocument> {
  throw new Error("Not implemented");
}
