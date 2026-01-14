import { StandardModel } from "@/api/sdf/dal/standard_model";
import { FuncId } from "@/api/sdf/dal/func";
import { Prop, PropId } from "@/api/sdf/dal/prop";
import { GroupedOptions } from "@/components/SelectMenu.vue";

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
  View = "view",
  ConfigurationFrameDown = "configurationFrameDown",
  ConfigurationFrameUp = "configurationFrameUp",
  AggregationFrame = "aggregationFrame",
}
interface Socket {
  name: string;
  annotations: Array<{ tokens: [] }>;
  arity: "many" | "one";
}

export type OutputSocketId = string;

export interface OutputSocket extends Socket {
  id: OutputSocketId;
  eligibleToReceiveData: boolean;
}

export type InputSocketId = string;

export interface InputSocket extends Socket {
  id: InputSocketId;
  eligibleToSendData: boolean;
}

export interface UninstalledVariant {
  schemaId: string;
  schemaName: string;
  displayName: string | null;
  category: string;
  color: string;
  componentType: ComponentType;
  link: string | null;
  description: string | null;
}

export interface SchemaVariant {
  id: string;
  schemaVariantId: string;
  schemaName: string;
  schemaDocLinks?: string;
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
}

export const outputSocketsAndPropsFor = (schemaVariant: SchemaVariant): GroupedOptions => {
  const socketOptions = schemaVariant.outputSockets
    .map((socket) => ({
      label: `Output Socket: ${socket.name}`,
      value: `s_${socket.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));

  const opts = groupedPropsFor(schemaVariant);
  opts["Output Sockets"] = socketOptions;
  return opts;
};

export const groupedPropsFor = (schemaVariant: SchemaVariant): GroupedOptions => {
  const rootPropOptions = schemaVariant.props
    .filter(
      (p) =>
        p.eligibleToSendData &&
        (p.path === "/root/code" ||
          p.path === "/root/deleted_at" ||
          p.path === "/root/domain" ||
          p.path === "/root/qualification" ||
          p.path === "/root/resource" ||
          p.path === "/root/resource_value" ||
          p.path === "/root/secrets" ||
          p.path === "/root/si"),
    )
    .map((p) => ({
      label: p.name,
      value: `p_${p.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));

  // TODO(nick): collapse this logic into one iterator. This is relatively inexpensive, but is
  // still more wasteful than it needs to be.
  const codePropOptions = schemaVariant.props
    .filter((p) => p.eligibleToSendData && p.path.startsWith("/root/code/"))
    .map((p) => ({
      label: p.name,
      value: `p_${p.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));
  const domainPropOptions = schemaVariant.props
    .filter((p) => p.eligibleToSendData && p.path.startsWith("/root/domain/"))
    .map((p) => ({
      label: p.name,
      value: `p_${p.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));
  const qualificationPropOptions = schemaVariant.props
    .filter((p) => p.eligibleToSendData && p.path.startsWith("/root/qualification/"))
    .map((p) => ({
      label: p.name,
      value: `p_${p.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));
  const resourcePropOptions = schemaVariant.props
    .filter((p) => p.eligibleToSendData && p.path.startsWith("/root/resource/"))
    .map((p) => ({
      label: p.name,
      value: `p_${p.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));
  const resourceValuePropOptions = schemaVariant.props
    .filter((p) => p.eligibleToSendData && p.path.startsWith("/root/resource_value/"))
    .map((p) => ({
      label: p.name,
      value: `p_${p.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));
  const secretsPropOptions = schemaVariant.props
    .filter((p) => p.eligibleToSendData && p.path.startsWith("/root/secrets/"))
    .map((p) => ({
      label: p.name,
      value: `p_${p.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));
  const siPropOptions = schemaVariant.props
    .filter((p) => p.eligibleToSendData && p.path.startsWith("/root/si/"))
    .map((p) => ({
      label: p.name,
      value: `p_${p.id}`,
    }))
    .sort((a, b) => a.label.localeCompare(b.label));

  return {
    "root/": rootPropOptions,
    "root/code/": codePropOptions,
    "root/domain/": domainPropOptions,
    "root/qualification/": qualificationPropOptions,
    "root/resource/": resourcePropOptions,
    "root/resource_value/": resourceValuePropOptions,
    "root/secrets/": secretsPropOptions,
    "root/si/": siPropOptions,
  };
};

export const inputSocketsAndPropsFor = (schemaVariant: SchemaVariant): GroupedOptions => {
  const opts = groupedPropsFor(schemaVariant);
  opts["Input Sockets"] = rawInputSocketsFor(schemaVariant);
  return opts;
};

export const inputSocketsFor = (schemaVariant: SchemaVariant): GroupedOptions => {
  return {
    "Input Sockets": rawInputSocketsFor(schemaVariant),
  };
};

const rawInputSocketsFor = (schemaVariant: SchemaVariant) => {
  return schemaVariant.inputSockets.map((socket) => ({
    label: socket.name,
    value: `s_${socket.id}`,
  }));
};

export const findSchemaVariantForPropOrSocketId = (
  schemaVariants: SchemaVariant[],
  propId: PropId | null | undefined,
  outputSocketId: OutputSocketId | null | undefined,
) => {
  if (propId && outputSocketId) throw new Error("Either prop or output socket");

  return schemaVariants.find((sv) => {
    if (propId && sv.props.map((p) => p.id).includes(propId)) return sv;
    if (outputSocketId && sv.outputSockets.map((o) => o.id).includes(outputSocketId)) return sv;
    return undefined;
  });
};
