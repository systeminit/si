import _ from "lodash";
import * as Rx from "rxjs";
import { diagramSchemaVariants$ } from "@/observable/diagram";

export type DiagramKind = "configuration";

export type DiagramProviderMetadata = string;

export interface DiagramOutputProvider {
  id: number;
  ty: DiagramProviderMetadata;
  color: number;
}

export interface DiagramOutputSocket {
  id: number;
  name: string;
  diagramKind: DiagramKind;
  provider: DiagramOutputProvider;
}

export interface DiagramInputProvider {
  id: number;
  ty: DiagramProviderMetadata;
  color: number;
}

export interface DiagramInputSocket {
  id: number;
  name: string;
  diagramKind: DiagramKind;
  provider: DiagramInputProvider;
}

export interface DiagramSchemaVariant {
  id: number;
  name: string;
  schemaName: string;
  color: number;
  inputSockets: DiagramInputSocket[];
  outputSockets: DiagramOutputSocket[];
}

export interface DiagramNodeKindComponent {
  kind: DiagramKind;
  componentId: number;
}

export type DiagramNodeKind = DiagramNodeKindComponent;

export interface DiagramNodePosition {
  diagramKind: DiagramKind;
  systemId?: number;
  x: number;
  y: number;
}

export interface DiagramNodeTemplate {
  name: string;
  title: string;
  kind: DiagramKind;
  schemaVariantId: number;
}

export interface DiagramNode {
  id: number;
  name: string;
  title: string;
  kind: DiagramNodeKind;
  schemaVariantId: number;
  positions: DiagramNodePosition[];
}
export type DiagramNodes = Array<DiagramNode>;

export type DiagramSchemaVariants = Array<DiagramSchemaVariant>;

export async function variantById(id: number): Promise<DiagramSchemaVariant> {
  const variants = await Rx.firstValueFrom(diagramSchemaVariants$);
  if (!variants) throw new Error("variants not found");

  const variant = variants.find((v) => v.id === id);
  if (!variant) throw Error(`schema variant not found: ${id}`);
  return variant;
}

export async function outputSocketById(
  id: number,
): Promise<DiagramOutputSocket> {
  const variants = await Rx.firstValueFrom(diagramSchemaVariants$);
  if (!variants) throw new Error("variants not found");

  for (const variant of variants) {
    for (const socket of variant.outputSockets) {
      if (socket.id === id) {
        return socket;
      }
    }
  }
  throw new Error(`output socket not found: ${id}`);
}

export async function inputSocketById(id: number): Promise<DiagramInputSocket> {
  const variants = await Rx.firstValueFrom(diagramSchemaVariants$);
  if (!variants) throw new Error("variants not found");

  for (const variant of variants) {
    for (const socket of variant.inputSockets) {
      if (socket.id === id) {
        return socket;
      }
    }
  }
  throw new Error(`input socket not found: ${id}`);
}

export function inputSocketByVariantAndProvider(
  schemaVariant: DiagramSchemaVariant,
  providerMetadata: DiagramProviderMetadata,
): DiagramInputSocket {
  const socket = schemaVariant.inputSockets.find((socket) =>
    _.isEqual(socket.provider.ty, providerMetadata),
  );
  if (!socket) throw new Error("source schema variant not found");
  return socket;
}
