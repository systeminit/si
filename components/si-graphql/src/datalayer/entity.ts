import uuidv4 from "uuid/v4";

import { Component } from "@/datalayer/component";
import { cdb } from "@/db";

export interface EntityData {
  id?: string;
  name: string;
  description: string;
  componentId?: string;
  workspaceId: string;
  userId: string;
  supportedActions?: string[];
  __typename?: string;
  nodeType?: string;
  isEntity?: boolean;
}

export interface EntityObject extends EntityData {
  fqId?(): string;
  save?(): Promise<void>;
}

export interface Entity<T extends EntityObject> {
  New(a: EntityConstructorArgs<T>): T;
  getById(id: string): Promise<T>;
  getByFqId(id: string): Promise<T>;
  getByName(name: string): Promise<T>;
  getAll(): Promise<T[]>;
  hasOneComponent(args: HasOneComponentArgs<Component<any>>): void;
  hasOne(args: HasOneArgs<Entity<T>>): void;
  hasMany(args: HasManyArgs<Entity<T>>): void;
}

type EntityConstructorArgs<T> = T;

export interface HasOneComponentArgs<T extends Component<any>> {
  from: string;
  to: {
    field: string;
    model: T;
  };
}

export interface HasOneArgs<T extends Entity<any>> {
  from: string;
  to: {
    field: string;
    model: T;
  };
}

export interface HasManyArgs<T extends Entity<any>> {
  from: {
    __typename: string;
    field: string;
  };
  to: {
    field: string;
    model: T;
  };
}

export interface EntityArgs {
  nodeType: string;
  __typename: string;
  fqKey: string;
}

export function CreateEntity<T extends EntityObject>({
  __typename,
  nodeType,
  fqKey,
}: EntityArgs): Entity<T> {
  // These are the functions that will wind up on every instance
  //
  const instanceFunctions = {
    fqId(): string {
      return `${fqKey}:${this.id}`;
    },
    async save(): Promise<void> {
      const col = cdb.bucket.defaultCollection();
      return await col.upsert(this.fqId(), this);
    },
  };

  const hasOneComponent = function<F extends Component<any>>(
    args: HasOneComponentArgs<F>,
  ): void {
    instanceFunctions[args.to.field] = async function(): Promise<F> {
      return args.to.model.getByFqId(this[args.from]);
    };
  };

  const hasOne = function<F extends Entity<any>>(args: HasOneArgs<F>): void {
    instanceFunctions[args.to.field] = async function(): Promise<F> {
      return args.to.model.getByFqId(this[args.from]);
    };
  };

  const hasMany = function<F extends Entity<any>>(args: HasManyArgs<F>): void {
    instanceFunctions[args.to.field] = async function(): Promise<F[]> {
      const list: F[] = [];
      const col = cdb.bucket.defaultCollection();
      const results = await col.query(
        `SELECT * FROM si WHERE __typename = "${args.from.__typename}" AND ${
          args.from.field
        } = "${this.fqId()}"`,
      );
      for (const row of results.rows) {
        list.push(args.to.model.New(row.si));
      }
      return list;
    };
  };

  // The factory method that will create new instances
  const New = function New(data: EntityConstructorArgs<T>): EntityObject & T {
    const defaults = { __typename, nodeType, isEntity: true };
    if (!data.id) {
      defaults["id"] = uuidv4();
    }
    const entity = Object.assign(defaults, data, instanceFunctions);
    return entity;
  };

  // Getting things by ID
  const getById = async function getById(
    id: string,
  ): Promise<EntityObject & T> {
    const col = cdb.bucket.defaultCollection();
    const d = await col.get(`${fqKey}:${id}`);
    return New(d.value);
  };

  const getByFqId = async function getByFqId(
    fqId: string,
  ): Promise<EntityObject & T> {
    const col = cdb.bucket.defaultCollection();
    const d = await col.get(fqId);
    return New(d.value);
  };

  const getByName = async function getByName(
    name: string,
  ): Promise<EntityObject & T> {
    const col = cdb.bucket.defaultCollection();
    const results = await col.query(
      `SELECT * FROM si WHERE __typename = "${__typename}" AND name = "${name}"`,
    );
    if (results.rows.length == 0) {
      throw `Cannot find ${__typename} named ${name}`;
    }
    const i = results.rows[0].si;
    return New(i);
  };

  const getAll = async function getAll(): Promise<T[]> {
    const result: T[] = [];
    const col = cdb.bucket.defaultCollection();
    const results = await col.query(
      `SELECT * FROM si WHERE __typename = "${__typename}"`,
    );
    for (const row of results.rows) {
      result.push(New(row.si));
    }
    return result;
  };

  return {
    New,
    getById,
    getByFqId,
    getByName,
    hasOneComponent,
    hasOne,
    hasMany,
    getAll,
  };
}
