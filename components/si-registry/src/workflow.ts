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

// NOTE: Eventually, this needs to be expanded to allow for totally arbitrary selection
// criteria. For now, though, we can just rely on the fact that there is always a root
// selected node when we trigger the worfklow!
export interface Selector {
  types?: string[];
  fromProperty?: string[];
  depth?: "immediate" | "all";
  edgeKind?: "configures" | "deployment" | "includes";
  direction?: "input" | "output";
}

export interface ForEach {
  edgeKind?: "configures" | "deployment" | "includes";
  direction?: "input" | "output";
  depth?: "immediate" | "all";
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
  forEach?: ForEach;
}

export interface StepAction extends StepBase {
  kind: StepKind.Action;
  inputs: {
    name: VariableScalar;
  };
  failIfMissing?: VariableBool;
  selector?: Selector;
  strategy?: VariableScalar;
  forEach?: ForEach;
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
  ],
};

export const applicationDeploy: Workflow = {
  name: "application:deploy",
  kind: WorkflowKind.Action,
  title: "Application Deployment",
  description: "Deploy application",
  steps: [
    {
      kind: StepKind.Action,
      inputs: {
        name: { kind: VariableKind.String, value: "deploy" },
      },
      selector: {
        edgeKind: "includes",
        depth: "immediate",
        direction: "output",
        types: ["service"],
      },
    },
  ],
};

export const serviceDeploy: Workflow = {
  name: "service:deploy",
  kind: WorkflowKind.Action,
  title: "Service Deployment",
  description: "Deploy services",
  steps: [
    {
      kind: StepKind.Action,
      inputs: {
        name: { kind: VariableKind.String, value: "deploy" },
      },
      selector: {
        edgeKind: "deployment",
        depth: "immediate",
        direction: "output",
      },
    },
    {
      kind: StepKind.Action,
      inputs: {
        name: { kind: VariableKind.String, value: "deploy" },
      },
      selector: {
        fromProperty: ["implementation"],
      },
      forEach: {
        edgeKind: "deployment",
        depth: "immediate",
        direction: "output",
      },
    },
  ],
};

export const kubernetesServiceDeploy: Workflow = {
  name: "kubernetesService:deploy",
  kind: WorkflowKind.Action,
  title: "Kubernetes Service Deployment Implementation",
  description: "Deploy Kubernetes Objects",
  steps: [
    // This is where the kubernetes deployment order will go. See
    // next section!
  ],
};

// This order taken from Helm.
// https://github.com/helm/helm/blob/484d43913f97292648c867b56768775a55e4bba6/pkg/releaseutil/kind_sorter.go/
//
// It is published under the Apache license.
const k8sTypes = [
  "k8sNamespace",
  "k8sNetworkPolicy",
  "k8sResourceQuota",
  "k8sLimitRange",
  "k8sPodSecurityPolicy",
  "k8sPodDisruptionBudget",
  "k8sSecret",
  "k8sConfigMap",
  "k8sStorageClass",
  "k8sPersistentVolume",
  "k8sPersistentVolumeClaim",
  "k8sServiceAccount",
  "k8sCustomResourceDefinition",
  "k8sClusterRole",
  "k8sClusterRoleList",
  "k8sClusterRoleBinding",
  "k8sClusterRoleBindingList",
  "k8sRole",
  "k8sRoleList",
  "k8sRoleBinding",
  "k8sRoleBindingList",
  "k8sService",
  "k8sDaemonSet",
  "k8sPod",
  "k8sReplicationController",
  "k8sReplicaSet",
  "k8sDeployment",
  "k8sHorizontalPodAutoscaler",
  "k8sStatefulSet",
  "k8sJob",
  "k8sCronJob",
  "k8sIngress",
  "k8sAPIService",
];
for (const k8sType of k8sTypes) {
  kubernetesServiceDeploy.steps.push({
    kind: StepKind.Action,
    inputs: {
      name: { kind: VariableKind.String, value: "apply" },
    },
    selector: {
      edgeKind: "configures",
      depth: "all",
      direction: "input",
      types: [k8sType],
    },
  });
}

// Oh shit, it's the same!
export const kubernetesClusterDeploy: Workflow = {
  name: "kubernetesCluster:deploy",
  kind: WorkflowKind.Action,
  title: "Kubernetes Cluster Deployment",
  description: "Deploy a Kubernetes Cluster",
  steps: [
    {
      kind: StepKind.Action,
      inputs: {
        name: { kind: VariableKind.String, value: "deploy" },
      },
      selector: {
        edgeKind: "deployment",
        depth: "immediate",
        direction: "output",
      },
    },
    {
      kind: StepKind.Action,
      inputs: {
        name: { kind: VariableKind.String, value: "deploy" },
      },
      selector: {
        fromProperty: ["implementation"],
      },
    },
  ],
};

export const kubernetesApply: Workflow = {
  name: "kubernetesApply",
  kind: WorkflowKind.Action,
  title: "Kubernetes Apply",
  description: "Apply some stuff to a Kubernetes Cluster",
  steps: [
    {
      kind: StepKind.Command,
      inputs: {
        name: { kind: VariableKind.String, value: "apply" },
      },
    },
  ],
};

export const workflows: Record<string, Workflow> = {
  serviceDeploy,
  applicationDeploy,
  kubernetesServiceDeploy,
  kubernetesClusterDeploy,
  universalDeploy,
  kubernetesApply,
};
