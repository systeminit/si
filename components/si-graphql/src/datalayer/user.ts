import Model from "@/db";
import { Workspace } from "./workspace";
import { IntegrationInstance } from "./integration";

export class User extends Model {
  public readonly id!: string;
  public email!: string;
  public name?: string;
  public createdWorkspaces: Workspace[];
  public workspaces: Workspace[];
  public integrationInstances: IntegrationInstance[];

  public static tableName = "users";

  public static relationMappings = {
    createdWorkspaces: {
      relation: Model.HasManyRelation,
      modelClass: Workspace,
      join: {
        from: "users.id",
        to: "workspaces.creator_id",
      },
    },
    integrationInstances: {
      relation: Model.HasManyRelation,
      modelClass: IntegrationInstance,
      join: {
        from: "users.id",
        to: "integration_instances.user_id",
      },
    },
    workspaces: {
      relation: Model.ManyToManyRelation,
      modelClass: Workspace,
      join: {
        from: "users.id",
        through: {
          from: "users_workspaces.user_id",
          to: "users_workspaces.workspace_id",
        },
        to: "workspaces.id",
      },
    },
  };

  public static async createOrReturn(
    email: string,
    name: string,
  ): Promise<User> {
    let user = await User.query().findOne({ email });
    if (user === undefined) {
      user = await User.query().insertAndFetch({
        email,
        name,
      });
    }
    return user;
  }
}
