import { graphqlMutation } from "./apollo";
import { ChangeSet } from "./graphql-types";

interface ChangeSetConstructor {
  name: string;
  workspaceId: string;
  createdByUserId: string;
}

export class ChangeSetFactory {
  name: string;
  workspaceId: string;
  createdByUserId: string;

  constructor(args: ChangeSetConstructor) {
    this.name = args.name;
    this.workspaceId = args.workspaceId;
    this.createdByUserId = args.createdByUserId;
  }

  async create(count?: number): Promise<ChangeSet> {
    let variables;
    if (count) {
      variables = {
        name: `${this.name}${count}`,
        displayName: `${this.name}${count}`,
        workspaceId: this.workspaceId,
        createdByUserId: this.createdByUserId,
      };
    } else {
      variables = {
        name: `${this.name}`,
        displayName: `${this.name}`,
        workspaceId: this.workspaceId,
        createdByUserId: this.createdByUserId,
      };
    }
    const result = await graphqlMutation({
      typeName: "changeSet",
      methodName: "create",
      variables,
    });
    console.log("I got a changeset", { result });
    return result["item"];
  }

  async execute(changeSet: ChangeSet): Promise<ChangeSet> {
    const executedChangeSet = await graphqlMutation({
      typeName: "changeSet",
      methodName: "execute",
      variables: {
        id: changeSet.id,
      },
    });
    return executedChangeSet;
  }
}
