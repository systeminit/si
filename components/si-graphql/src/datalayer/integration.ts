import { RelationMappings } from "objection";

import { User } from "@/datalayer/user";
import { Workspace } from "@/datalayer/workspace";
import Model from "@/db";

export class Integration extends Model {
  public readonly id!: number;
  public name!: string;
  public description?: string;
  public options?: string;

  public static tableName = "integrations";
}

interface EnableOnWorkspaceResult {
  integrationInstance: IntegrationInstance;
  workspace: Workspace;
}

interface DisableOnWorkspaceResult {
  integrationInstance: IntegrationInstance;
  workspace: Workspace;
}

export class IntegrationInstance extends Model {
  public readonly id!: number;
  public name!: string;
  public description!: string;
  public options!: string;

  public user!: User;
  public integration!: Integration;
  public workspaces!: Workspace[];

  public static tableName = "integration_instances";

  public static get relationMappings(): RelationMappings {
    return {
      user: {
        relation: Model.BelongsToOneRelation,
        modelClass: User,
        join: {
          from: "integration_instances.user_id",
          to: "users.id",
        },
      },
      integration: {
        relation: Model.BelongsToOneRelation,
        modelClass: Integration,
        join: {
          from: "integration_instances.integration_id",
          to: "integrations.id",
        },
      },
      workspaces: {
        relation: Model.ManyToManyRelation,
        modelClass: Workspace,
        join: {
          from: "integration_instances.id",
          through: {
            from: "integration_instance_workspaces.integration_instance_id",
            to: "integration_instance_workspaces.workspace_id",
          },
          to: "workspaces.id",
        },
      },
    };
  }

  public static async enableOnWorkspace(
    integrationInstanceId,
    workspaceId,
    user,
  ): Promise<EnableOnWorkspaceResult> {
    let integrationInstance = await IntegrationInstance.query()
      .where("user_id", user.id)
      .andWhere("id", integrationInstanceId)
      .first();

    let workspace = await Workspace.query()
      .where("creator_id", user.id)
      .andWhere("id", workspaceId)
      .first();

    let exists = await integrationInstance
      .$relatedQuery("workspaces")
      .where("workspace_id", workspace.id);

    if (exists.length === 0) {
      await integrationInstance.$relatedQuery("workspaces").relate(workspace);
    }

    return {
      integrationInstance,
      workspace,
    };
  }

  public static async disableOnWorkspace(
    integrationInstanceId,
    workspaceId,
    user,
  ): Promise<DisableOnWorkspaceResult> {
    let integrationInstance = await IntegrationInstance.query()
      .where("user_id", user.id)
      .andWhere("id", integrationInstanceId)
      .first();

    let workspace = await Workspace.query()
      .where("creator_id", user.id)
      .andWhere("id", workspaceId)
      .first();

    let exists = await integrationInstance
      .$relatedQuery("workspaces")
      .where("workspace_id", workspace.id);

    if (exists.length === 1) {
      await integrationInstance
        .$relatedQuery("workspaces")
        .unrelate()
        .where("workspace_id", workspace.id);
    }

    return {
      integrationInstance,
      workspace,
    };
  }

  public static async getIntegrationInstances(
    user: User,
  ): Promise<IntegrationInstance[]> {
    return IntegrationInstance.query().where("user_id", user.id);
  }

  public static async deleteIntegrationInstance(
    id: number,
    user: User,
  ): Promise<IntegrationInstance> {
    let integrationInstance = await IntegrationInstance.query()
      .where("id", id)
      .where("user_id", user.id)
      .first();
    await IntegrationInstance.query()
      .delete()
      .where("id", id)
      .where("user_id", user.id);
    return integrationInstance;
  }

  public static async createIntegrationInstance(
    integrationId,
    name,
    description,
    options,
    user,
  ): Promise<IntegrationInstance> {
    let integrationInstance = await IntegrationInstance.query().insertAndFetch({
      //@ts-ignore It does exist, you bastards
      name,
      description,
      options,
      //eslint-disable-next-line
      integration_id: integrationId,
      //eslint-disable-next-line
      user_id: user.id,
    });
    return integrationInstance;
  }
}

export class IntegrationInstanceWorkspace extends Model {
  public readonly id!: number;

  public workspace!: Workspace;
  public integrationInstance!: IntegrationInstance;

  public static tableName = "integration_instance_workspaces";

  public static get relationMappings(): RelationMappings {
    return {
      integrationInstance: {
        relation: Model.BelongsToOneRelation,
        modelClass: IntegrationInstance,
        join: {
          from: "integration_instance_workspaces.integration_instance_id",
          to: "integration_instances.id",
        },
      },
      workspace: {
        relation: Model.BelongsToOneRelation,
        modelClass: Workspace,
        join: {
          from: "integration_instance_workspaces.workspace_id",
          to: "workspaces.id",
        },
      },
    };
  }
}
