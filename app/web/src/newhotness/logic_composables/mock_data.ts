import { ActionId, ActionKind, ActionState } from "@/api/sdf/dal/action";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ActionProposedView } from "../types";
import { debugConsoleLog } from "./debug";

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
        name: `${id} mock ${kind} action`,
        description: `${id} mock ${kind} action`,
        kind: kind as ActionKind,
        originatingChangeSetId: changeSetId,
        state: state as ActionState,
        myDependencies: [],
        dependentOn: [],
        holdStatusInfluencedBy: [],
        componentSchemaName: `${id} mock schema ${kind}`,
        componentName: `${id} mock component ${kind}`,
      });
      id++;
    }
  }

  // sets of actions with dependencies
  for (const state of [ActionState.Queued, ActionState.OnHold]) {
    const parent = {
      id: `${id}`,
      prototypeId: `${id}`,
      componentId: `${id}`,
      name: `${id} mock parent action`,
      description: `${id} mock parent action`,
      kind: ActionKind.Create,
      originatingChangeSetId: changeSetId,
      state,
      myDependencies: [] as ActionId[],
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: `${id} mock parent schema`,
      componentName: `${id} mock parent component`,
    };
    out.push(parent);
    id++;
    Object.values(ActionKind).forEach((kind) => {
      const child = {
        id: `${id}`,
        prototypeId: `${id}`,
        componentId: `${id}`,
        name: `${id} mock child action ${kind}`,
        description: `${id} mock child action ${kind}`,
        kind: kind as ActionKind,
        originatingChangeSetId: changeSetId,
        state,
        myDependencies: [] as string[],
        dependentOn: [parent.id] as ActionId[],
        holdStatusInfluencedBy: [],
        componentSchemaName: `${id} mock child schema`,
        componentName: `${id} mock child component`,
      };
      parent.myDependencies.push(child.id);
      out.push(child);
      id++;
      if (kind === ActionKind.Update) {
        const child2 = {
          id: `${id}`,
          prototypeId: `${id}`,
          componentId: `${id}`,
          name: `${id} mock child child action`,
          description: `${id} mock child child action`,
          kind: ActionKind.Create,
          originatingChangeSetId: changeSetId,
          state,
          myDependencies: [],
          dependentOn: [parent.id] as ActionId[],
          holdStatusInfluencedBy: [],
          componentSchemaName: `${id} mock child child schema`,
          componentName: `${id} mock child child component`,
        };
        child.myDependencies.push(child2.id);
        out.push(child2);
        id++;
      }
    });
  }

  debugConsoleLog(`${id - 1} mock actions generated`);

  return out;
};
