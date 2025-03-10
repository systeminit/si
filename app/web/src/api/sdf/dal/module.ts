import { SchemaId, SchemaVariantId } from "./schema";

export type ModuleId = string;

export interface LatestModule {
  id: ModuleId;
  name: string;
  description: string;
  ownerUserId: string;
  ownerDisplayName: string;
  metadata: object;
  latestHash: string;
  latestHashCreatedAt: IsoDateString;
  schemaId: SchemaId;
}

export interface ModuleContributeRequest {
  name: string;
  version: string;
  schemaVariantId: SchemaVariantId;
  isPrivateModule: boolean;
}
