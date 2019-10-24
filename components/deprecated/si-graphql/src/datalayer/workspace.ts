import uuidv4 from "uuid/v4";
import map from "lodash/map";
import union from "lodash/union";
import pull from "lodash/pull";

import { cdb } from "@/db";
import { User } from "@/datalayer/user";
import {
  IntegrationInstance,
  IntegrationInstanceInterface,
} from "@/datalayer/integration";

interface WorkspaceInterface {
  id: string;
  name: string;
  description?: string;
  __typename: string;
  creatorId: string;
  memberIds: string[];
  integrationInstanceIds: string[];
}

export class Workspace implements WorkspaceInterface {
  public readonly id!: string;
  public name!: string;
  public description?: string;
  public creatorId!: string;
  public memberIds!: string[];
  public integrationInstanceIds: string[] = [];

  public __typename = "Workspace";

  constructor({
    name,
    description,
    creator,
    workspace,
  }: {
    id?: string;
    name?: string;
    description?: string;
    creator?: User;
    workspace?: WorkspaceInterface;
  }) {
    if (workspace !== undefined) {
      for (const key of Object.keys(workspace)) {
        this[key] = workspace[key];
      }
    } else {
      this.id = uuidv4();
      this.name = name;
      this.description = description;
      this.creatorId = `user:${creator.id}`;
      this.memberIds = [`user:${creator.id}`];
      this.integrationInstanceIds = [];
    }
  }

  public get fqId(): string {
    return `workspace:${this.id}`;
  }

  public static async getById(id: string): Promise<Workspace> {
    const col = cdb.bucket.defaultCollection();
    const wsremote = await col.get(`workspace:${id}`);
    return new Workspace({ workspace: wsremote.value });
  }

  public static async getWorkspacesCreatedByUser(
    user: User,
  ): Promise<Workspace[]> {
    const col = cdb.bucket.defaultCollection();
    const results = await col.query(
      `SELECT * FROM si WHERE __typename = "Workspace" AND creatorId = "${user.fqId}"`,
    );
    return map(results.rows, w => {
      return new Workspace({ workspace: w.si });
    });
  }

  public static async getWorkspacesForUser(user: User): Promise<Workspace[]> {
    const col = cdb.bucket.defaultCollection();
    const results = await col.query(
      `SELECT * FROM si WHERE __typename = "Workspace" AND ("${user.fqId}" IN memberIds)`,
    );
    return map(results.rows, w => {
      return new Workspace({ workspace: w.si });
    });
  }

  public static async create({
    name,
    description,
    creator,
  }: {
    name: string;
    description: string;
    creator: User;
  }): Promise<Workspace> {
    const ws = new Workspace({ name, description, creator });
    const col = cdb.bucket.defaultCollection();
    await col.insert(`workspace:${ws.id}`, ws);
    return ws;
  }

  public static async delete({
    workspaceId,
    user,
  }: {
    workspaceId: string;
    user: User;
  }): Promise<Workspace> {
    const col = cdb.bucket.defaultCollection();
    const ws = await col.get(`workspace:${workspaceId}`);
    if (ws.value.creatorId == user.fqId) {
      await col.remove(`workspace:${workspaceId}`);
    } else {
      throw "You can only delete workspaces you created";
    }
    return new Workspace({ workspace: ws.value });
  }

  public static async enableIntegrationInstance(
    workspaceId: string,
    integrationInstanceId: string,
  ): Promise<[IntegrationInstance, Workspace]> {
    const workspace = await Workspace.getById(workspaceId);
    return workspace.enableIntegrationInstance(integrationInstanceId);
  }

  public static async disableIntegrationInstance(
    workspaceId: string,
    integrationInstanceId: string,
  ): Promise<[IntegrationInstance, Workspace]> {
    const workspace = await Workspace.getById(workspaceId);
    return workspace.disableIntegrationInstance(integrationInstanceId);
  }

  public async creator(): Promise<User> {
    return User.getByFqId(this.creatorId);
  }

  public async members(): Promise<User[]> {
    const col = cdb.bucket.defaultCollection();
    const memberList: User[] = [];

    for (const memberId of this.memberIds) {
      const member = await col.get(memberId);
      memberList.push(new User({ user: member.value }));
    }
    return memberList;
  }

  public async integrationInstances(): Promise<IntegrationInstance[]> {
    const col = cdb.bucket.defaultCollection();
    const list: IntegrationInstance[] = [];

    for (const id of this.integrationInstanceIds) {
      const result = await col.get(id);
      const i: IntegrationInstanceInterface = result.value;
      list.push(new IntegrationInstance({ integrationInstance: i }));
    }
    return list;
  }

  public async save(): Promise<Workspace> {
    const col = cdb.bucket.defaultCollection();
    return await col.upsert(this.fqId, this);
  }

  public async enableIntegrationInstance(
    integrationInstanceId: string,
  ): Promise<[IntegrationInstance, Workspace]> {
    const integrationInstance = await IntegrationInstance.getById(
      integrationInstanceId,
    );
    this.integrationInstanceIds = union(this.integrationInstanceIds, [
      integrationInstance.fqId,
    ]);
    this.save();
    return [integrationInstance, this];
  }

  public async disableIntegrationInstance(
    integrationInstanceId: string,
  ): Promise<[IntegrationInstance, Workspace]> {
    const integrationInstance = await IntegrationInstance.getById(
      integrationInstanceId,
    );
    pull(this.integrationInstanceIds, integrationInstance.fqId);
    this.save();
    return [integrationInstance, this];
  }
}
