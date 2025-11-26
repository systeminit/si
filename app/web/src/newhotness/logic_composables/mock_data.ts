import { ActionId, ActionKind, ActionState } from "@/api/sdf/dal/action";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ActionProposedView } from "../types";

export const generateMockActions = (changeSetId: ChangeSetId) => {
  let id = 1;
  const out = [] as ActionProposedView[];

  // one independent action of each kind in each state
  for (const kind in ActionKind) {
    for (const state in ActionState) {
      out.push({
        id: `${id}`,
        prototypeId: `${id}`,
        componentId: `${id}`,
        name: `mock ${kind} action`,
        description: `mock ${kind} action`,
        kind: kind as ActionKind,
        originatingChangeSetId: changeSetId,
        state: state as ActionState,
        myDependencies: [],
        dependentOn: [],
        holdStatusInfluencedBy: [],
        componentSchemaName: `mock schema ${kind}`,
        componentName: `mock component ${kind}`,
      });
      id++;
    }
  }

  // a set of actions with dependencies
  const parent = {
    id: `${id}`,
    prototypeId: `${id}`,
    componentId: `${id}`,
    name: `mock parent action`,
    description: `mock parent action`,
    kind: ActionKind.Create,
    originatingChangeSetId: changeSetId,
    state: ActionState.Queued,
    myDependencies: [] as ActionId[],
    dependentOn: [],
    holdStatusInfluencedBy: [],
    componentSchemaName: `mock parent schema`,
    componentName: `mock parent component`,
  };
  out.push(parent);
  id++;
  for (const kind in ActionKind) {
    const child = {
      id: `${id}`,
      prototypeId: `${id}`,
      componentId: `${id}`,
      name: `mock child action ${kind}`,
      description: `mock child action ${kind}`,
      kind: kind as ActionKind,
      originatingChangeSetId: changeSetId,
      state: ActionState.Queued,
      myDependencies: [],
      dependentOn: [parent.id] as ActionId[],
      holdStatusInfluencedBy: [],
      componentSchemaName: `mock child schema`,
      componentName: `mock child component`,
    };
    parent.myDependencies.push(child.id);
    out.push(child);
    id++;
  }

  return out;
};
