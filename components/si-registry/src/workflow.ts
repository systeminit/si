import { Prop } from "./registryEntry";

export enum WorkflowKind {
  Action = "action",
  Top = "top",
}

export interface WorkflowBase {
  name: string;
  kind: WorkflowKind;
  title: string;
  description: string;
  steps: Step[];
}

export interface WorkflowForAction extends WorkflowBase {
  args?: never;
}

export interface WorkflowTop extends WorkflowBase {
  args: Prop[];
}

export type Workflow = WorkflowTop | WorkflowForAction;

export enum VariableKind {
  String = "string",
  Number = "number",
  Bool = "bool",
  Array = "array",
  Object = "object",
  Args = "args",
  Context = "context",
  Output = "output",
  Store = "store",
}

export interface VariableBase {
  kind: VariableKind;
  value: unknown;
}

export interface VariableBool extends VariableBase {
  kind: VariableKind.Bool;
  value: boolean;
}

export interface VariableString extends VariableBase {
  kind: VariableKind.String;
  value: string;
}

export interface VariableNumber extends VariableBase {
  kind: VariableKind.Number;
  value: number;
}

export interface VariableArgs {
  kind: VariableKind.Args;
  path: string[];
}

export interface VariableContext {
  kind: VariableKind.Context;
  path: string[];
}

export interface VariableOutput {
  kind: VariableKind.Output;
  path: string[];
}

export interface VariableStore {
  kind: VariableKind.Store;
  path: string[];
}

export interface VariableArray {
  kind: VariableKind.Array;
  value: Variable[] | VariableRef;
}

export interface VariableObject {
  kind: VariableKind.Object;
  value: { [key: string]: Variable } | VariableRef;
}

export type Variable =
  | VariableString
  | VariableNumber
  | VariableArgs
  | VariableContext
  | VariableOutput
  | VariableStore;

export type VariableRef =
  | VariableArgs
  | VariableOutput
  | VariableStore
  | VariableContext;

export type VariableScalar =
  | VariableString
  | VariableNumber
  | VariableBool
  | VariableRef;

export enum OrderByDirection {
  ASC = "asc",
  DESC = "desc",
}

export enum BooleanTerm {
  And = "and",
  Or = "or",
}

export enum Comparison {
  Equals = "equals",
  NotEquals = "notEquals",
  Contains = "contains",
  Like = "like",
  NotLike = "notLike",
}

export enum FieldType {
  String = "string",
  Int = "int",
  Boolean = "boolean",
}

export interface Expression {
  field: string;
  value: VariableScalar;
  comparison: Comparison;
  fieldType: FieldType;
}

export interface Item {
  query?: Query;
  expression?: Expression;
}

export interface Query {
  booleanTerm?: BooleanTerm;
  isNot?: boolean;
  items: Item[];
}

export interface FilterItem {
  filter?: Filter;
  expression?: Expression;
}

export interface Filter {
  booleanTerm?: BooleanTerm;
  isNot?: boolean;
  filters: FilterItem[];
}

export enum StepKind {
  Command = "command",
  Action = "action",
  Workflow = "workflow",
}

export interface Selector {
  query?: Query;
  byId?: VariableScalar;
  filter?: Filter;
  depth?: "immediate" | "all" | "none";
  edgeKind?: "configures" | "deployment" | string;
}

export interface StepBase {
  kind: StepKind;
  inputs?: unknown;
  outputs?: unknown;
  storeOutput?: string[];
}

export interface StepCommand extends StepBase {
  kind: StepKind.Command;
  inputs: {
    name: VariableScalar;
    args?: VariableArray;
  };
  failIfMissing?: VariableBool;
  selector?: Selector;
  strategy?: VariableScalar;
}

export interface StepAction extends StepBase {
  kind: StepKind.Action;
  inputs: {
    name: VariableScalar;
  };
  failIfMissing?: VariableBool;
  selector?: Selector;
  strategy?: VariableScalar;
}

export interface StepWorkflow extends StepBase {
  kind: StepKind.Workflow;
  inputs: {
    name: VariableScalar;
    args: VariableObject;
  };
  failIfMissing?: VariableBool;
  selector?: Selector;
  strategy?: VariableScalar;
}

export type Step = StepCommand | StepAction | StepWorkflow;

// * Workflows can target global state
// * Workflows can be built for a specific slot, which means they don't take arguments (they inherit the slot)
// * Workflows that aren't a specific slot can take arguments, and they might user provided
// * Workflows can call actions, actions only take arguments that are specific to the intent - (shutdown -h/-r)
// * Workflows are assigned to slots on entities by class, or by specific entity
// * Actions update resource state
// * Reactions can watch resource state and trigger workflows (either on a slot or global if the global workflow doesn't require user input)
// * Workflows can be executed by the user either directly for a global workflow or you entity slot directly or by a reaction
// * Entities have default implementations of their workflow slots, so you don't specify it all
// * Context to the workflow is always the real context of where it was triggered

export const universalDeploy: Workflow = {
  name: "universal:deploy",
  kind: WorkflowKind.Action,
  title: "Universal Deploy",
  description: "Deploy Things!! so fun!",
  steps: [
    {
      kind: StepKind.Command,
      inputs: {
        name: { kind: VariableKind.String, value: "universal:deploy" },
      },
      strategy: { kind: VariableKind.String, value: "linear" },
      failIfMissing: { kind: VariableKind.Bool, value: false },
    },
    //{
    //  kind: StepKind.Action,
    //  inputs: {
    //    name: { kind: VariableKind.String, value: "universal:deploy" },
    //  },
    //  selector: {
    //    byId: {
    //      kind: VariableKind.Context,
    //      path: ["entity", "id"],
    //    },
    //    //query: {
    //    //  items: [
    //    //    {
    //    //      expression: {
    //    //        field: "id",
    //    //        value: {
    //    //          kind: VariableKind.Context,
    //    //          path: ["entity", "id"],
    //    //        },
    //    //        comparison: Comparison.Equals,
    //    //        fieldType: FieldType.String,
    //    //      },
    //    //    },
    //    //  ],
    //    //},
    //    depth: "immediate",
    //    edgeKind: "configures",
    //  },
    //  strategy: { kind: VariableKind.String, value: "linear" },
    //  failIfMissing: { kind: VariableKind.Bool, value: false },
    //},
  ],
};

export const workflows: Record<string, Workflow> = {
  universalDeploy,
};
