import uuidv4 from "uuid/v4";

import { cdb } from "@/db";
import { Integration } from "@/datalayer/integration";

export interface ComponentData {
  id?: string;
  __typename?: string;
  nodeType?: string;
  name: string;
  description: string;
  rawDataJson: string;
  integrationId: string;
  supportedActions: string[];
  isComponent?: boolean;
}

export interface ComponentObject extends ComponentData {
  fqId?(): string;
  integration?(): Promise<Integration>;
  save?(): Promise<void>;
}

export interface Component<T extends ComponentObject> {
  New(a: ComponentConstructorArgs<T>): T;
  getById(id: string): Promise<T>;
  getByFqId(id: string): Promise<T>;
  getByName(name: string): Promise<T>;
  getAll(): Promise<T[]>;
  hasOne(args: HasOneArgs<Component<any>>): void;
  hasMany(args: HasManyArgs<Component<any>>): void;
}

type ComponentConstructorArgs<T> = T;

export interface HasOneArgs<T extends Component<any>> {
  from: string;
  to: {
    field: string;
    model: T;
  };
}

export interface HasManyArgs<T extends Component<any>> {
  from: {
    __typename: string;
    field: string;
  };
  to: {
    field: string;
    model: T;
  };
}

export interface ComponentArgs {
  nodeType: string;
  __typename: string;
  fqKey: string;
}

export function CreateComponent<T extends ComponentObject>({
  __typename,
  nodeType,
  fqKey,
}: ComponentArgs): Component<T> {
  // These are the functions that will wind up on every instance
  //
  const componentFunctions = {
    fqId(): string {
      return `${fqKey}:${this.id}`;
    },
    async integration(): Promise<Integration> {
      return Integration.getByFqId(this.integrationId);
    },
    async save(): Promise<void> {
      const col = cdb.bucket.defaultCollection();
      return await col.upsert(this.fqId(), this);
    },
  };

  const hasOne = function<F extends Component<any>>(args: HasOneArgs<F>): void {
    componentFunctions[args.to.field] = async function(): Promise<F> {
      return args.to.model.getByFqId(this[args.from]);
    };
  };

  const hasMany = function<F extends Component<any>>(
    args: HasManyArgs<F>,
  ): void {
    componentFunctions[args.to.field] = async function(): Promise<F[]> {
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
  const New = function New(
    data: ComponentConstructorArgs<T>,
  ): ComponentObject & T {
    const defaults = { __typename, nodeType, isComponent: true };
    if (!data.id) {
      defaults["id"] = uuidv4();
    }
    const component = Object.assign(
      { __typename, nodeType, isComponent: true },
      data,
      componentFunctions,
    );
    return component;
  };

  // Getting things by ID
  const getById = async function getById(
    id: string,
  ): Promise<ComponentObject & T> {
    const col = cdb.bucket.defaultCollection();
    const d = await col.get(`${fqKey}:${id}`);
    return New(d.value);
  };

  const getByFqId = async function getByFqId(
    fqId: string,
  ): Promise<ComponentObject & T> {
    const col = cdb.bucket.defaultCollection();
    const d = await col.get(fqId);
    return New(d.value);
  };

  const getByName = async function getByName(
    name: string,
  ): Promise<ComponentObject & T> {
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
    hasOne,
    hasMany,
    getAll,
  };
}
