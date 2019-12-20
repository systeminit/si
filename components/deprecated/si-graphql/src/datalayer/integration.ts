import map from "lodash/map";
import uuidv4 from "uuid/v4";

import { cdb } from "@/db";
import { User } from "@/datalayer/user";
import { Workspace } from "@/datalayer/workspace";

interface IntegrationOptions {
  [key: string]: any; //eslint-disable-line
}

export class Integration {
  public readonly id!: string;
  public name!: string;
  public description?: string;
  public options?: IntegrationOptions;
  public image!: string;
  public __typename = "Integration";

  constructor({
    id,
    name,
    description,
    image,
    options,
  }: {
    id?: string;
    name: string;
    description: string;
    image: string;
    options?: IntegrationOptions;
  }) {
    if (id === undefined) {
      this.id = uuidv4();
    } else {
      this.id = id;
    }
    this.name = name;
    this.description = description;
    this.image = image;
    if (options !== undefined) {
      this.options = options;
    }
  }

  public get fqId(): string {
    return `integration:${this.id}`;
  }

  public async upsert(): Promise<Integration> {
    const col = cdb.bucket.defaultCollection();
    await col.upsert(this.fqId, this);
    return this;
  }

  public static async getById(id: string): Promise<Integration> {
    const col = cdb.bucket.defaultCollection();
    const i = await col.get(`integration:${id}`);
    return new Integration(i.value);
  }

  public static async getByFqId(fqId: string): Promise<Integration> {
    const col = cdb.bucket.defaultCollection();
    const i = await col.get(fqId);
    return new Integration(i.value);
  }

  public static async getByName(name: string): Promise<Integration> {
    const col = cdb.bucket.defaultCollection();
    const results = await col.query(
      `SELECT * FROM si WHERE __typename = "Integration" AND name = "${name}"`,
    );
    const i = results.rows[0].si;
    return new Integration(i);
  }

  public static async getAll(): Promise<Integration[]> {
    const col = cdb.bucket.defaultCollection();
    const results = await col.query(
      `SELECT * FROM si WHERE __typename = "Integration"`,
    );
    return map(results.rows, i => {
      return new Integration(i.si);
    });
  }
}

export interface IntegrationInstanceInterface {
  id: string;
  name: string;
  description: string;
  options: string;
  userId: string;
  integrationId: string;
  __typename: string;
}

export class IntegrationInstance implements IntegrationInstanceInterface {
  public readonly id!: string;
  public name!: string;
  public description!: string;
  public options!: string;
  public userId: string;
  public integrationId: string;

  public __typename = "IntegrationInstance";

  constructor({
    name,
    description,
    options,
    user,
    integrationId,
    integrationInstance,
  }: {
    name?: string;
    description?: string;
    options?: string;
    user?: User;
    integrationId?: string;
    integrationInstance?: IntegrationInstanceInterface;
  }) {
    if (integrationInstance !== undefined) {
      for (const key of Object.keys(integrationInstance)) {
        this[key] = integrationInstance[key];
      }
    } else {
      this.id = uuidv4();
      this.name = name;
      this.description = description;
      this.options = options;
      this.userId = user.fqId;
      this.integrationId = `integration:${integrationId}`;
    }
  }

  public get fqId(): string {
    return `integration:instance:${this.id}`;
  }

  public async user(): Promise<User> {
    return User.getByFqId(this.userId);
  }

  public async integration(): Promise<Integration> {
    return Integration.getByFqId(this.integrationId);
  }

  public async workspaces(): Promise<Workspace[]> {
    const col = cdb.bucket.defaultCollection();
    const results = await col.query(
      `SELECT * FROM si WHERE __typename = "Workspace" AND ("${this.fqId}" IN integrationInstanceIds)`,
    );
    return map(results.rows, r => {
      return new Workspace({ workspace: r.si });
    });
  }

  public static async create({
    integrationId,
    name,
    description,
    options,
    user,
  }: {
    integrationId: string;
    name: string;
    description: string;
    options: string;
    user: User;
  }): Promise<IntegrationInstance> {
    const integrationInstance = new IntegrationInstance({
      integrationId,
      name,
      description,
      options,
      user,
    });
    const col = cdb.bucket.defaultCollection();
    await col.insert(integrationInstance.fqId, integrationInstance);
    return integrationInstance;
  }

  public static async delete({
    id,
    user,
  }: {
    id: string;
    user: User;
  }): Promise<IntegrationInstance> {
    const col = cdb.bucket.defaultCollection();
    const result = await col.get(`integration:instance:${id}`);
    if (result.value.userId == user.fqId) {
      // Clean up the reference to any enabled instance on a workspace
      await col.query(
        `UPDATE si SET integrationInstanceIds=ARRAY_REMOVE(integrationInstanceIds, "integration:instance:${id}") WHERE __typename = "Workspace"`,
      );
      await col.remove(`integration:instance:${id}`);
    } else {
      throw "You can only delete integration instances you created";
    }
    return new IntegrationInstance({ integrationInstance: result.value });
  }

  public static async getForUser(user: User): Promise<IntegrationInstance[]> {
    const col = cdb.bucket.defaultCollection();
    const results = await col.query(
      `SELECT * FROM si WHERE __typename = "IntegrationInstance" AND userId = "${user.fqId}"`,
    );
    return map(results.rows, r => {
      return new IntegrationInstance({ integrationInstance: r.si });
    });
  }

  public static async getById(
    id: string,
    user?: User,
  ): Promise<IntegrationInstance> {
    const col = cdb.bucket.defaultCollection();
    const result = await col.get(`integration:instance:${id}`);
    if (user !== undefined && result.value.userId != user.fqId) {
      throw "Cannot access integration instances you didn't create";
    }
    return new IntegrationInstance({ integrationInstance: result.value });
  }
}
