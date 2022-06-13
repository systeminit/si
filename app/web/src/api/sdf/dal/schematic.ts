import _ from "lodash";
import * as Rx from "rxjs";
import { schematicSchemaVariants$ } from "@/observable/schematic";

export enum SchematicKind {
  Deployment = "deployment",
  Component = "component",
}

export function schematicKindFromString(s: string): SchematicKind {
  switch (s) {
    case "deployment":
      return SchematicKind.Deployment;
    case "component":
      return SchematicKind.Component;
  }
  throw Error(`Unknown SchematicKind member: ${s}`);
}

export interface EditorContext {
  applicationNodeId: number;
  systemId?: number;
}

export type SchematicProviderMetadata = string;

export interface SchematicOutputProvider {
  id: number;
  ty: SchematicProviderMetadata;
  color: number;
}

export interface SchematicOutputSocket {
  id: number;
  name: string;
  schematicKind: SchematicKind;
  provider: SchematicOutputProvider;
}

export interface SchematicInputProvider {
  id: number;
  ty: SchematicProviderMetadata;
  color: number;
}

export interface SchematicInputSocket {
  id: number;
  name: string;
  schematicKind: SchematicKind;
  provider: SchematicInputProvider;
}

export interface SchematicSchemaVariant {
  id: number;
  name: string;
  schemaName: string;
  color: number;
  inputSockets: SchematicInputSocket[];
  outputSockets: SchematicOutputSocket[];
}

export interface SchematicNodeComponentKind {
  kind: "component";
  componentId: number;
}

export interface SchematicNodeDeploymentKind {
  kind: "deployment";
  componentId: number;
}

//export interface SchematicNodeSystemKind {
//  kind: "system";
//}

export type SchematicNodeKind =
  | SchematicNodeComponentKind
  //| SchematicNodeSystemKind;
  | SchematicNodeDeploymentKind;

export interface SchematicNodePosition {
  deploymentNodeId?: number;
  schematicKind: SchematicKind;
  systemId?: number;
  x: number;
  y: number;
}

export interface SchematicNodeTemplate {
  name: string;
  title: string;
  kind: "component" | "deployment";
  schemaVariantId: number;
}

export interface SchematicNode {
  id: number;
  name: string;
  title: string;
  kind: SchematicNodeKind;
  schemaVariantId: number;
  positions: SchematicNodePosition[];
}
export type SchematicNodes = Array<SchematicNode>;

export interface SchematicConnection {
  destinationNodeId: number;
  destinationSocketId: number;
  sourceNodeId: number;
  sourceSocketId: number;
}
export type SchematicConnections = Array<SchematicConnection>;

export type SchematicSchemaVariants = Array<SchematicSchemaVariant>;

export interface Schematic {
  nodes: SchematicNodes;
  connections: SchematicConnections;
}

export async function variantById(id: number): Promise<SchematicSchemaVariant> {
  const variants = await Rx.firstValueFrom(schematicSchemaVariants$);
  if (!variants) throw new Error("variants not found");

  const variant = variants.find((v) => v.id === id);
  if (!variant) throw Error("schema variant not found: " + id);
  return variant;
}

export async function outputSocketById(
  id: number,
): Promise<SchematicOutputSocket> {
  const variants = await Rx.firstValueFrom(schematicSchemaVariants$);
  if (!variants) throw new Error("variants not found");

  for (const variant of variants) {
    for (const socket of variant.outputSockets) {
      if (socket.id === id) {
        return socket;
      }
    }
  }
  throw new Error("output socket not found: " + id);
}

export async function inputSocketById(
  id: number,
): Promise<SchematicInputSocket> {
  const variants = await Rx.firstValueFrom(schematicSchemaVariants$);
  if (!variants) throw new Error("variants not found");

  for (const variant of variants) {
    for (const socket of variant.inputSockets) {
      if (socket.id === id) {
        return socket;
      }
    }
  }
  throw new Error("input socket not found: " + id);
}

export function inputSocketByVariantAndProvider(
  schemaVariant: SchematicSchemaVariant,
  providerMetadata: SchematicProviderMetadata,
): SchematicInputSocket {
  const socket = schemaVariant.inputSockets.find((socket) =>
    _.isEqual(socket.provider.ty, providerMetadata),
  );
  if (!socket) throw new Error("source schema variant not found");
  return socket;
}
