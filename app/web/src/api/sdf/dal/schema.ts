import { StandardModel } from "@/api/sdf/dal/standard_model";
import { FuncId } from "@/api/sdf/dal/func";
import { Prop, PropId } from "@/api/sdf/dal/prop";

export enum SchemaKind {
  Concept = "concept",
  Implementation = "implementation",
  Concrete = "concrete",
}

export interface Schema extends StandardModel {
  name: string;
  kind: SchemaKind;
  ui_menu_name: string;
  ui_menu_category: string;
  ui_hidden: boolean;
}

export type SchemaVariantId = string;
export type SchemaId = string;

export enum ComponentType {
  Component = "component",
  ConfigurationFrameDown = "configurationFrameDown",
  ConfigurationFrameUp = "configurationFrameUp",
  AggregationFrame = "aggregationFrame",
}

export type OutputSocketId = string;

export interface OutputSocket {
  id: OutputSocketId;
  name: string;
  eligibleToReceiveData: boolean;
}

export type InputSocketId = string;

export interface InputSocket {
  id: InputSocketId;
  name: string;
  eligibleToSendData: boolean;
}

export interface SchemaVariant {
  schemaVariantId: string;
  schemaName: string;
  displayName: string | null;
  category: string;
  color: string;
  componentType: ComponentType;
  link: string | null;
  description: string | null;

  created_at: IsoDateString;
  updated_at: IsoDateString;

  version: string;
  assetFuncId: FuncId;
  funcIds: FuncId[];
  isLocked: boolean;

  schemaId: SchemaId;

  inputSockets: InputSocket[];
  outputSockets: OutputSocket[];
  props: Prop[];
  canCreateNewComponents: boolean;

  canContribute: boolean;
  canUpdate: boolean;
}

export const outputSocketsAndPropsFor = (schemaVariant: SchemaVariant) => {
  const socketOptions = schemaVariant.outputSockets.map((socket) => ({
    label: `Output Socket: ${socket.name}`,
    value: `s_${socket.id}`,
  }));

  // output
  const propOptions = schemaVariant.props
    .filter((p) => p.eligibleToReceiveData)
    .map((p) => ({
      label: `Attribute: ${p.path}`,
      value: `p_${p.id}`,
    }));
  return { socketOptions, propOptions };
};

export const inputSocketsAndPropsFor = (schemaVariant: SchemaVariant) => {
  const socketOptions = schemaVariant.inputSockets.map((socket) => ({
    label: `Input Socket: ${socket.name}`,
    value: `s_${socket.id}`,
  }));

  const propOptions = schemaVariant.props
    .filter((p) => p.eligibleToSendData)
    .map((p) => ({
      label: `Attribute: ${p.path}`,
      value: `p_${p.id}`,
    }));

  return { socketOptions, propOptions };
};

export const findSchemaVariantForPropOrSocketId = (
  schemaVariants: SchemaVariant[],
  propId: PropId | null | undefined,
  outputSocketId: OutputSocketId | null | undefined,
) => {
  if (propId && outputSocketId) throw new Error("Either prop or output socket");

  return schemaVariants.find((sv) => {
    if (propId && sv.props.map((p) => p.id).includes(propId)) return sv;
    if (
      outputSocketId &&
      sv.outputSockets.map((o) => o.id).includes(outputSocketId)
    )
      return sv;
    return undefined;
  });
};
