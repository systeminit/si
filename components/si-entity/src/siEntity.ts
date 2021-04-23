import _ from "lodash";
import {
  Prop,
  findProp,
  ItemProp,
  PropString,
  PropNumber,
  PropArray,
  registry,
} from "si-registry";

import { SiStorable } from "./siStorable";
import { validate, ValidateResult } from "./validation";
import {
  ItemPropString,
  ItemPropNumber,
  ItemPropArray,
  PropObject,
  ItemPropObject,
  RegistryEntry,
} from "si-registry/dist/registryEntry";

export interface ISiEntity {
  id: string;
  nodeId: string;
  name: string;
  description: string;
  entityType: string;
  ops: Op[];
  tombstones: OpTombstone[];
  arrayMeta: {
    [path: string]: ArrayMeta;
  };
  properties: {
    [systemId: string]: Record<string, any>;
  };
  siStorable?: SiStorable;
}

export interface ArrayMeta {
  length: number;
}

export enum OpType {
  Set = "set",
  Unset = "unset",
  Tombstone = "tombstone",
}

export enum OpSource {
  Manual = "manual",
  Expression = "expression",
  Inferred = "inferred",
}

interface OpBase {
  op: OpType;
  source: OpSource;
  path: string[];
  value?: unknown;
  system: "baseline" | string;
}

export interface OpSet extends OpBase {
  op: OpType.Set;
  path: string[];
  value: string | number | boolean | null;
}

export interface OpUnset extends OpBase {
  op: OpType.Unset;
  path: string[];
  value?: never;
}

export interface OpTombstone extends OpBase {
  op: OpType.Tombstone;
  path: string[];
  value?: never;
  source: OpSource;
}

export type Op = OpSet | OpUnset | OpTombstone;

export type RegistryPropertyPath = string[];

export interface Setters {
  set(input: Omit<OpSet, "source" | "op">): ValidateResult;
}

export interface EditField {
  type: string;
  widgetName: string;
  schema: Prop | ItemProp;
  name: string;
  path: string[];
}

export interface EditFieldString extends EditField {
  type: "string";
  schema: PropString | ItemPropString;
}

export interface EditFieldNumber extends EditField {
  type: "number";
  schema: PropNumber | ItemPropNumber;
}

export interface EditFieldArray extends EditField {
  type: "array";
  schema: PropArray | ItemPropArray;
}

export interface EditFieldObject extends EditField {
  type: "object";
  schema: PropObject | ItemPropObject;
}

export class SiEntity implements ISiEntity {
  id: string;
  nodeId: string;
  name: string;
  description: string;
  entityType: ISiEntity["entityType"];
  ops: Op[];
  tombstones: OpTombstone[];
  arrayMeta: {
    [path: string]: ArrayMeta;
  };
  properties: {
    [systemId: string]: Record<string, any>;
  };
  siStorable?: ISiEntity["siStorable"];

  constructor({ entityType }: { entityType: string }) {
    this.id = "fake";
    this.nodeId = "fake";
    this.name = "fake";
    this.description = "fake";
    this.entityType = entityType;
    this.ops = [];
    this.tombstones = [];
    this.arrayMeta = {};
    this.properties = {};
  }

  static fromJson(input: ISiEntity): SiEntity {
    const entity = new SiEntity(input);
    entity.id = input.id;
    entity.nodeId = input.nodeId;
    entity.name = input.name;
    entity.description = input.description;
    entity.entityType = input.entityType;
    entity.ops = input.ops;
    entity.tombstones = input.tombstones;
    entity.arrayMeta = input.arrayMeta;
    entity.properties = input.properties;
    entity.siStorable = input.siStorable;
    return entity;
  }

  fullPropPath(op: OpBase): string[] {
    return _.concat([this.entityType], op.path);
  }

  validateProp(op: OpSet): ValidateResult {
    const result = validate(this.fullPropPath(op), `${op.value}`);
    return result;
  }

  findProp(op: OpSet): Prop | undefined {
    const result = findProp(this.fullPropPath(op));
    return result;
  }

  pathToString(path: string[]): string {
    return path.join(".");
  }

  updateArrayMetaLength(op: OpSet): void {
    const fullPath = [this.entityType].concat(op.path);
    for (let x = 1; x < fullPath.length; x++) {
      const pathToCheck = fullPath.slice(0, x + 1);
      const prop = findProp(pathToCheck);
      if (prop && prop.type == "array") {
        if (x != fullPath.length - 1) {
          const lookAheadProp = fullPath[x + 1];
          const arrayIndex = _.toNumber(lookAheadProp);
          if (!_.isNaN(arrayIndex)) {
            const arrayMetaKey = this.pathToString(pathToCheck);
            if (this.arrayMeta[arrayMetaKey]) {
              const lastIndex = this.arrayMeta[arrayMetaKey].length - 1;
              if (arrayIndex > lastIndex) {
                const difference = arrayIndex - lastIndex;
                this.arrayMeta[arrayMetaKey].length =
                  this.arrayMeta[arrayMetaKey].length + difference;
              }
            } else {
              this.arrayMeta[arrayMetaKey] = {
                length: arrayIndex + 1,
              };
            }
          }
        }
      }
    }
  }

  subPath(a: string[], b: string[]): boolean {
    let checkPath;
    if (a.length == b.length) {
      checkPath = a;
    } else if (a.length > b.length) {
      checkPath = a.slice(0, b.length);
    } else {
      return false;
    }
    return _.isEqual(checkPath, b);
  }

  isTombstoned(op: {
    op?: OpSet["op"];
    value?: OpSet["value"];
    path: OpSet["path"];
    source: OpSet["source"];
    system: OpSet["system"];
  }): boolean {
    for (const tombstone of this.tombstones) {
      if (
        this.subPath(op.path, tombstone.path) &&
        op.source == tombstone.source &&
        op.system == tombstone.system
      ) {
        return true;
      }
    }
    return false;
  }

  addOpSet(op: OpSet): ValidateResult {
    const result = this.validateProp(op);
    if (result.errors) {
      return result;
    }
    this.updateArrayMetaLength(op);
    _.remove(
      this.ops,
      (p) =>
        p.op == OpType.Set &&
        p.system == op.system &&
        p.source == op.source &&
        _.isEqual(p.path, op.path),
    );
    this.ops.push(op);
    return { success: true };
  }

  decrementArrayMetaLength(op: OpUnset): void {
    const arrayPath = [this.entityType].concat(
      op.path.slice(0, op.path.length - 1),
    );
    const checkPath = this.pathToString(arrayPath);
    if (this.arrayMeta[checkPath]) {
      this.arrayMeta[checkPath].length = this.arrayMeta[checkPath].length - 1;
    }
    if (this.arrayMeta[checkPath].length == 0) {
      delete this.arrayMeta[checkPath];
    }
  }

  addOpUnset(op: OpUnset): void {
    _.remove(this.ops, (p) => {
      const shouldRemove =
        p.op == OpType.Set &&
        p.system == op.system &&
        p.source == op.source &&
        this.subPath(p.path, op.path);
      return shouldRemove;
    });

    // If what we are removing is an index itself, we need to
    // renumber all the other items.
    if (!_.isNaN(_.toNumber(op.path[op.path.length - 1]))) {
      _.forEach(this.ops, (p) => {
        if (p.system == op.system) {
          let checkPath;
          if (p.path.length >= op.path.length) {
            checkPath = p.path.slice(0, op.path.length - 1);
          } else {
            return;
          }
          if (_.isEqual(checkPath, op.path.slice(0, op.path.length - 1))) {
            const pIndex = _.toNumber(p.path[op.path.length - 1]);
            const metaIndex = _.toNumber(op.path[op.path.length - 1]);
            if (pIndex > metaIndex) {
              p.path[op.path.length - 1] = `${pIndex - 1}`;
            }
          }
        }
      });
      this.decrementArrayMetaLength(op);
    }
  }

  addOpTombstone(op: OpTombstone): void {
    if (_.find(this.tombstones, op)) {
      return;
    } else {
      this.tombstones.push(op);
    }
  }

  removeOpTombstone(op: OpTombstone): void {
    _.remove(this.tombstones, op);
  }

  isOverridden(op: OpSet, targetSystem: string): boolean {
    if (op.source == OpSource.Inferred && op.system == "baseline") {
      const override = _.find(this.ops, (p) => {
        // The item we are checking is never overriden.
        if (_.isEqual(p, op)) {
          return false;
        }
        //
        return (
          _.isEqual(p.path, op.path) &&
          ((p.source == OpSource.Manual && p.system == "baseline") ||
            p.system == targetSystem) &&
          !this.isTombstoned(p as OpSet)
        );
      });
      if (override) {
        return true;
      }
    } else if (op.source == OpSource.Inferred && op.system != "baseline") {
      const override = _.find(this.ops, (p) => {
        if (_.isEqual(p, op)) {
          return false;
        }
        return (
          _.isEqual(p.path, op.path) &&
          p.system == op.system &&
          op.system == targetSystem &&
          !this.isTombstoned(p as OpSet)
        );
      });
      if (override) {
        return true;
      }
    } else if (op.source == OpSource.Manual && op.system == "baseline") {
      const override = _.find(this.ops, (p) => {
        if (_.isEqual(p, op)) {
          return false;
        }
        return (
          _.isEqual(p.path, op.path) &&
          p.source == OpSource.Manual &&
          p.system == targetSystem &&
          !this.isTombstoned(p as OpSet)
        );
      });
      if (override) {
        return true;
      }
    }
    return false;
  }

  _fixupAndWalkArray(arr: any[]): any[] {
    arr = _.filter(arr, (i) => !_.isUndefined(i));
    for (let x = 0; x < arr.length; x++) {
      if (_.isArray(arr[x])) {
        arr[x] = this._fixupAndWalkArray(arr[x]);
      } else if (_.isObjectLike(arr[x])) {
        arr[x] = this._fixupArraysWalkObject(arr[x]);
      }
    }
    return arr;
  }

  _fixupArraysWalkObject(obj: Record<string, any>): Record<string, any> {
    for (const key of Object.keys(obj)) {
      if (_.isArray(obj[key])) {
        obj[key] = this._fixupAndWalkArray(obj[key]);
      } else if (_.isObjectLike(obj[key])) {
        obj[key] = this._fixupArraysWalkObject(obj[key]);
      }
    }
    return obj;
  }

  fixupArrays(newProperties: SiEntity["properties"]): SiEntity["properties"] {
    for (const system of Object.keys(newProperties)) {
      newProperties[system] = this._fixupArraysWalkObject(
        newProperties[system],
      );
    }
    return newProperties;
  }

  computeProperties(): void {
    const systems = _.uniq(
      _.filter(_.map(this.ops, "system"), (s) => s != "baseline"),
    );
    const newProperties: SiEntity["properties"] = { baseline: {} };
    for (const system of systems) {
      newProperties[system] = {};
    }

    for (const op of this.ops) {
      if (op.op == OpType.Set) {
        if (op.system == "baseline") {
          for (const system of systems) {
            if (!this.isOverridden(op, system) && !this.isTombstoned(op)) {
              _.set(newProperties[system], op.path, _.cloneDeep(op.value));
            }
          }
        }
        if (!this.isOverridden(op, op.system) && !this.isTombstoned(op)) {
          _.set(newProperties[op.system], op.path, _.cloneDeep(op.value));
        }
      }
    }

    this.properties = this.fixupArrays(newProperties);
  }

  set({
    source,
    system,
    path,
    value,
  }: {
    source: OpSet["source"];
    system: OpSet["system"];
    path: OpSet["path"];
    value: OpSet["value"];
  }): ValidateResult {
    const op: OpSet = {
      op: OpType.Set,
      source,
      system,
      path,
      value,
    };
    return this.addOpSet(op);
  }

  getProperty<T>({
    system,
    path,
  }: {
    system?: OpSet["system"];
    path: OpSet["path"];
  }): T {
    if (this.properties[system]) {
      return _.get(this.properties[system], path);
    } else {
      return _.get(this.properties["baseline"], path);
    }
  }

  inferred(): Setters {
    const result: Setters = {
      set(input: Parameters<Setters["set"]>[0]): ValidateResult {
        return this.set({
          source: OpSource.Inferred,
          system: input.system,
          path: input.path,
          value: input.value,
        });
      },
    };
    return result;
  }

  schema(): RegistryEntry {
    return registry[this.entityType];
  }

  toEditField(
    editFields: EditField[],
    checkProp: {
      path: EditField["path"];
      schema: EditField["schema"];
    },
  ): void {
    let widgetName: string;
    if (checkProp.schema.widget) {
      widgetName = checkProp.schema.widget.name;
    } else if (checkProp.schema.type == "string") {
      widgetName = "text";
    } else if (checkProp.schema.type == "number") {
      widgetName = "number";
    } else if (checkProp.schema.type == "object") {
      widgetName = "header";
    } else if (checkProp.schema.type == "boolean") {
      widgetName = "checkbox";
    } else if (checkProp.schema.type == "map") {
      widgetName = "map";
    } else if (checkProp.schema.type == "array") {
      widgetName = "array";
    } else {
      widgetName = "unknown";
    }
    let name: string;
    if (checkProp.schema.displayName) {
      name = checkProp.schema.displayName;
    } else if (checkProp.schema.name) {
      name = checkProp.schema.name;
    }
    const editField: EditField = {
      type: checkProp.schema.type,
      schema: checkProp.schema,
      path: checkProp.path,
      widgetName,
      name,
    };
    editFields.push(editField);
    if (checkProp.schema.type == "object") {
      for (const p of checkProp.schema.properties) {
        const path = _.cloneDeep(checkProp.path);
        path.push(p.name);
        this.toEditField(editFields, { path, schema: p });
      }
    } else if (checkProp.schema.type == "array") {
      //const path = _.cloneDeep(checkProp.path);
      //const p = checkProp.schema.itemProperty;
      //path.push("a0");
      //this.toEditField(editFields, { path, schema: p });
    }
  }

  editFields(): EditField[] {
    const editFields: EditField[] = [];
    const rootSchema = registry[this.entityType];
    const toCheck: {
      path: EditField["path"];
      schema: EditField["schema"];
    }[] = _.map(rootSchema.properties, (p) => {
      return { path: [p.name], schema: p };
    });

    for (const checkProp of toCheck) {
      this.toEditField(editFields, checkProp);
    }
    return editFields;
  }

  arrayEditFields(editField: EditField, index: number): EditField[] {
    const editFields: EditField[] = [];
    if (editField.schema.type == "array") {
      const path = _.cloneDeep(editField.path);
      path.push(`${index}`);
      const rootSchema = editField.schema.itemProperty;
      if (rootSchema.type == "object") {
        const toCheck: {
          path: EditField["path"];
          schema: EditField["schema"];
        }[] = _.map(rootSchema.properties, (p) => {
          const subPath = _.cloneDeep(path);
          subPath.push(p.name);
          return { path: subPath, schema: p };
        });

        for (const checkProp of toCheck) {
          this.toEditField(editFields, checkProp);
        }
      } else {
        const path = _.cloneDeep(editField.path);
        path.push(`${index}`);
        this.toEditField(editFields, { path, schema: rootSchema });
      }
    }
    return editFields;
  }

  isPathTombstoned(path: OpTombstone["path"]): boolean {
    const tombstone = _.find(this.tombstones, ["path", path]);
    if (tombstone) {
      return true;
    } else {
      return false;
    }
  }

  valueOpForPath({
    path,
    system,
  }: {
    path: OpSet["path"];
    system: OpSet["system"];
  }): OpSet | undefined {
    const ops: OpSet[] = _.filter(this.ops, (o) =>
      _.isEqual(o.path, path),
    ) as OpSet[];
    if (ops.length) {
      let finalOp: OpSet;
      for (const op of ops) {
        if (
          op.system == system &&
          op.source == OpSource.Manual &&
          !this.isTombstoned(op)
        ) {
          return op as OpSet;
        } else if (
          op.system == system &&
          op.source == OpSource.Inferred &&
          !this.isTombstoned(op)
        ) {
          if (finalOp) {
            if (finalOp.system == "baseline") {
              finalOp = op as OpSet;
            }
          } else {
            finalOp = op as OpSet;
          }
        } else if (
          op.system == "baseline" &&
          op.source == OpSource.Manual &&
          !this.isTombstoned(op)
        ) {
          if (finalOp) {
            if (
              finalOp.system == "baseline" &&
              finalOp.source == OpSource.Inferred
            ) {
              finalOp = op as OpSet;
            }
          } else {
            finalOp = op as OpSet;
          }
        } else if (
          op.system == "baseline" &&
          op.source == OpSource.Inferred &&
          !this.isTombstoned(op)
        ) {
          if (!finalOp) {
            finalOp = op as OpSet;
          }
        }
      }
      return finalOp;
    } else {
      return undefined;
    }
  }

  valueFrom({
    path,
    system,
    source,
  }: {
    path: OpSet["path"];
    system: OpSet["system"];
    source: OpSet["source"];
  }): string | number | boolean | undefined {
    const op = _.find(
      this.ops,
      (o) =>
        o.source == source && _.isEqual(o.path, path) && o.system == system,
    );
    if (op) {
      return op.value;
    } else {
      return undefined;
    }
  }

  hasValueFrom({
    path,
    system,
    source,
  }: {
    path: OpSet["path"];
    system: OpSet["system"];
    source: OpSet["source"];
  }): boolean {
    const op = _.find(
      this.ops,
      (o) =>
        o.source == source && _.isEqual(o.path, path) && o.system == system,
    );
    if (op) {
      return true;
    } else {
      return false;
    }
  }
}
