import _ from "lodash";
import YAML, { LineCounter, Parser } from "yaml";
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
  CodeKind,
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
  code: {
    [systemId: string]: string;
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
  from?: {
    entityId: string;
    entityType: string;
    arrayRoot?: true;
  };
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

export enum CodeDecorationItemType {
  Gutter = "gutter",
  Line = "line",
}

export interface CodeDecorationItemBase {
  startLine: number;
  startCol: number;
  endLine: number;
  endCol: number;
}

export interface CodeDecorationItemDriven extends CodeDecorationItemBase {
  type: CodeDecorationItemType.Line;
  kind: "driven";
  source: OpSource;
  system: OpBase["system"];
}

export interface CodeDecorationItemChanged extends CodeDecorationItemBase {
  type: CodeDecorationItemType.Line;
  kind: "changed";
}

export interface CodeDecorationItemQualification
  extends CodeDecorationItemBase {
  type: CodeDecorationItemType.Line;
  kind: "qualification";
}

export type CodeDecorationItem =
  | CodeDecorationItemDriven
  | CodeDecorationItemChanged
  | CodeDecorationItemQualification;

export interface YamlDocPath {
  path: string[];
  line: number;
  col: number;
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
  code: {
    [systemId: string]: string;
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
    this.code = {};
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
    entity.code = input.code || {}; // whoops
    return entity;
  }

  fullPropPath(op: OpBase): string[] {
    return _.concat([this.entityType], op.path);
  }

  validateProp(op: OpSet): ValidateResult {
    let result;
    if (op.value == undefined) {
      result = validate(
        this.fullPropPath(op),
        `___UNDEFINEDMONSTERYARGBLARG___`,
      );
    } else {
      result = validate(this.fullPropPath(op), `${op.value}`);
    }
    return result;
  }

  findProp(op: OpSet): Prop | ItemProp | undefined {
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

  subPath(long: string[], short: string[]): boolean {
    let checkPath;
    if (long.length == short.length) {
      checkPath = long;
    } else if (long.length > short.length) {
      checkPath = long.slice(0, short.length);
    } else {
      return false;
    }
    return _.isEqual(checkPath, short);
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
    //const result = this.validateProp(op);
    //if (result.errors) {
    //  return result;
    //}

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

  setCode(
    source: OpSet["source"],
    system: OpSet["system"],
    code: string,
  ): void {
    if (this.schema().code?.kind == CodeKind.YAML) {
      this.setCodeYaml(source, system, code);
    }
  }

  setCodeYaml(
    source: OpSet["source"],
    system: OpSet["system"],
    code: string,
  ): void {
    let newData;
    try {
      newData = YAML.parse(code);
    } catch (e) {
      console.log("Failed to load valid yaml for code", e);
    }
    if (_.isArray(newData) || _.isString(newData) || _.isNumber(newData)) {
      return;
    }
    this._setCodeFromObject(source, system, [], newData);
  }

  _setCodeFromScalar(
    source: OpSet["source"],
    system: OpSet["system"],
    path: string[],
    value: string | number | boolean,
  ): void {
    const existingOp = this.valueOpForPath({ system, path });
    if (existingOp) {
      if (_.isEqual(existingOp.value, value)) {
        return;
      } else {
        this.set({ source, system, path, value });
      }
    } else {
      this.set({ source, system, path, value });
    }
  }

  _setCodeFromArray(
    source: OpSet["source"],
    system: OpSet["system"],
    startingPath: string[],
    target: any[],
  ): void {
    for (let x = 0; x < target.length; x++) {
      const path = _.cloneDeep(startingPath);
      path.push(`${x}`);
      const v = target[x];
      if (_.isArray(v)) {
        this._setCodeFromArray(source, system, path, v);
      } else if (_.isObjectLike(v)) {
        this._setCodeFromObject(source, system, path, v);
      } else {
        this._setCodeFromScalar(source, system, path, v);
      }
    }
  }

  _setCodeFromObject(
    source: OpSet["source"],
    system: OpSet["system"],
    startingPath: string[],
    targetObject: Record<string, any>,
  ): void {
    for (const k of Object.keys(targetObject)) {
      const path = _.cloneDeep(startingPath);
      path.push(k);
      if (_.isArray(targetObject[k])) {
        this._setCodeFromArray(source, system, path, targetObject[k]);
      } else if (_.isObjectLike(targetObject[k])) {
        this._setCodeFromObject(source, system, path, targetObject[k]);
      } else {
        this._setCodeFromScalar(source, system, path, targetObject[k]);
      }
    }
  }

  getCode(system: string): string | null {
    if (this.schema().code) {
      if (this.code[system]) {
        return this.code[system];
      } else {
        return this.code["baseline"];
      }
    } else {
      return null;
    }
  }

  _yamlDocToPathsBlockSeq(
    lineCounter: LineCounter,
    results: YamlDocPath[],
    startingPath: string[],
    blockSeq: YAML.CST.BlockSequence,
  ): YamlDocPath[] {
    for (let key = 0; key < blockSeq.items.length; key++) {
      const item = blockSeq.items[key];
      if (item.value?.type == "scalar") {
        const path = _.cloneDeep(startingPath);
        path.push(`${key}`);
        const { line, col } = lineCounter.linePos(item.value.offset);
        results.push({ path, line, col });
      } else if (item.value.type == "block-map") {
        const path = _.cloneDeep(startingPath);
        path.push(`${key}`);
        const newBlockMap = item.value;
        this._yamlDocToPathsBlockMap(lineCounter, results, path, newBlockMap);
      } else if (item.value.type == "block-seq") {
        const path = _.cloneDeep(startingPath);
        path.push(`${key}`);
        const newBlockSeq = item.value;
        this._yamlDocToPathsBlockSeq(lineCounter, results, path, newBlockSeq);
      }
    }
    return results;
  }

  _yamlDocToPathsBlockMap(
    lineCounter: LineCounter,
    results: YamlDocPath[],
    startingPath: string[],
    blockMap: YAML.CST.BlockMap,
  ): YamlDocPath[] {
    for (const item of blockMap.items) {
      if (item.value?.type == "scalar" && item.key?.type == "scalar") {
        const path = _.cloneDeep(startingPath);
        const key = item.key?.source;
        path.push(key);
        const { line, col } = lineCounter.linePos(item.value.offset);
        results.push({ path, line, col });
      } else if (item.value.type == "block-map" && item.key?.type == "scalar") {
        const path = _.cloneDeep(startingPath);
        const key = item.key?.source;
        path.push(key);
        const newBlockMap = item.value;
        this._yamlDocToPathsBlockMap(lineCounter, results, path, newBlockMap);
      } else if (item.value.type == "block-seq" && item.key?.type == "scalar") {
        const path = _.cloneDeep(startingPath);
        const key = item.key?.source;
        path.push(key);
        const newBlockSeq = item.value;
        this._yamlDocToPathsBlockSeq(lineCounter, results, path, newBlockSeq);
      }
    }
    return results;
  }

  _yamlDocToPaths(system: string): YamlDocPath[] {
    const lineCounter = new LineCounter();
    const parser = new Parser(lineCounter.addNewLine);
    const tokens = parser.parse(this.getCode(system));
    const tokenArray = Array.from(tokens);
    const result: YamlDocPath[] = [];
    if (tokenArray[0]?.type == "document") {
      const document = tokenArray[0];
      if (document.value?.type == "block-map") {
        this._yamlDocToPathsBlockMap(lineCounter, result, [], document.value);
      }
    }
    return result;
  }

  getCodeDecorations(
    system: string,
    diffPaths: string[][],
  ): CodeDecorationItem[] {
    const yamlDocPaths = this._yamlDocToPaths(system);
    const results: CodeDecorationItem[] = [];
    for (const op of this.ops) {
      if (op.op == OpType.Set) {
        const yamlDocPath = _.find(yamlDocPaths, { path: op.path });
        if (yamlDocPath) {
          results.push({
            kind: "driven",
            type: CodeDecorationItemType.Line,
            source: op.source,
            system: op.system,
            startLine: yamlDocPath.line,
            startCol: yamlDocPath.col,
            endLine: yamlDocPath.line,
            endCol: yamlDocPath.col,
          });
          const isDiff = _.find(diffPaths, (p) => _.isEqual(p, op.path));
          if (isDiff) {
            results.push({
              kind: "changed",
              type: CodeDecorationItemType.Line,
              startLine: yamlDocPath.line,
              startCol: yamlDocPath.col,
              endLine: yamlDocPath.line,
              endCol: yamlDocPath.col,
            });
          }
          const valid = this.validateProp(op);
          if (valid.errors) {
            results.push({
              kind: "qualification",
              type: CodeDecorationItemType.Line,
              startLine: yamlDocPath.line,
              startCol: yamlDocPath.col,
              endLine: yamlDocPath.line,
              endCol: yamlDocPath.col,
            });
          }
        }
      }
    }
    return results;
  }

  _yamlNumberReplacerArray(
    system: string,
    basePath: string[],
    targetObject: any[],
  ): void {
    for (let x = 0; x < targetObject.length; x++) {
      const path = _.cloneDeep(basePath);
      path.push(`${x}`);
      if (_.isArray(targetObject[x])) {
        this._yamlNumberReplacerArray(system, path, targetObject[x]);
      } else if (_.isObject(targetObject[x])) {
        this._yamlNumberReplacerObject(system, path, targetObject[x]);
      } else {
        const fakeOp: OpSet = {
          op: OpType.Set,
          path,
          value: targetObject[x],
          system,
          source: OpSource.Manual,
        };
        const prop = this.findProp(fakeOp);
        if (prop) {
          if (prop.type == "array" && prop.itemProperty.type == "number") {
            const numberValue = _.toNumber(targetObject[x]);
            if (!_.isNaN(numberValue)) {
              targetObject[x] = numberValue;
            }
          } else if (prop.type == "number") {
            const numberValue = _.toNumber(targetObject[x]);
            if (!_.isNaN(numberValue)) {
              targetObject[x] = numberValue;
            }
          }
        } else {
          console.log("cannot find prop", { fakeOp });
        }
      }
    }
  }

  _yamlNumberReplacerObject(
    system: string,
    basePath: string[],
    targetObject: Record<string, any>,
  ): void {
    for (const key in targetObject) {
      const path = _.cloneDeep(basePath);
      path.push(key);
      if (_.isArray(targetObject[key])) {
        this._yamlNumberReplacerArray(system, path, targetObject[key]);
      } else if (_.isObject(targetObject[key])) {
        this._yamlNumberReplacerObject(system, path, targetObject[key]);
      } else {
        const fakeOp: OpSet = {
          op: OpType.Set,
          path,
          value: targetObject[key],
          system,
          source: OpSource.Manual,
        };
        let prop = this.findProp(fakeOp);
        if (!prop) {
          const mapPath = path.slice(0, path.length - 1);
          fakeOp.path = mapPath;
          prop = this.findProp(fakeOp);
          console.log("looking for prop", { prop, fakeOp });
          if (prop && prop.type != "map") {
            continue;
          }
        }
        if (prop) {
          if (prop.type == "number") {
            const numberValue = _.toNumber(targetObject[key]);
            if (!_.isNaN(numberValue)) {
              targetObject[key] = numberValue;
            }
          } else if (prop.type == "map") {
            if (prop.valueProperty.type == "number") {
              const numberValue = _.toNumber(targetObject[key]);
              if (!_.isNaN(numberValue)) {
                targetObject[key] = numberValue;
              }
            }
          }
        } else {
          console.log("cannot find prop", { fakeOp });
        }
      }
    }
  }

  yamlNumberReplacer(): Record<string, any> {
    const numberified = _.cloneDeep(this.properties);
    for (const system of Object.keys(numberified)) {
      this._yamlNumberReplacerObject(system, [], numberified[system]);
    }
    return numberified;
  }

  //yamlNumberReplacer(): Record<string, any> {
  //  const numberified = _.cloneDeep(this.properties);
  //  for (const op of this.ops) {
  //    if (op.op == OpType.Set) {
  //      const prop = this.findProp(op);
  //      if (prop) {
  //        if (prop.type == "number") {
  //          for (const system of Object.keys(numberified)) {
  //            const value = _.get(numberified, [system, ...op.path]);
  //            if (value) {
  //              const number = _.toNumber(op.value);
  //              if (!_.isNaN(number)) {
  //                _.set(numberified, [system, ...op.path], number);
  //              }
  //            }
  //          }
  //        }
  //      } else {
  //        console.log("Cannot find prop for op", { op });
  //      }
  //    }
  //  }
  //  return numberified;
  //}

  computeCode(): void {
    const newCode: Record<string, string> = {};
    const schema = this.schema();
    const codeToGen = _.concat(["baseline"], Object.keys(this.properties));
    const numberifiedProperties = this.yamlNumberReplacer();
    if (schema.code && schema.code.kind == CodeKind.YAML) {
      for (const system of codeToGen) {
        const code = YAML.stringify(numberifiedProperties[system], {
          //["apiVersion", "kind", "metadata", "spec", "data"]
          // -- this is probably not the most efficient implementation
          // of this sort algorithm, but I'm getting tired. :)
          sortMapEntries: (a, b): number => {
            if (a.key == b.key) {
              return 0;
            } else if (a.key == "apiVersion") {
              return -1;
            } else if (b.key == "apiVersion") {
              return 1;
            } else if (a.key == "kind") {
              return -1;
            } else if (b.key == "kind") {
              return 1;
            } else if (a.key == "metadata") {
              return -1;
            } else if (b.key == "metadata") {
              return 1;
            } else if (a.key == "spec") {
              return -1;
            } else if (b.key == "spec") {
              return 1;
            } else if (a.key == "data") {
              return -1;
            } else if (b.key == "data") {
              return 1;
            } else {
              return a.key < b.key ? -1 : a.key > b.key ? 1 : 0;
            }
          },
        });
        newCode[system] = `---\n${code}`;
      }
    }
    this.code = newCode;
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
    this.computeCode();
  }

  set(args: {
    source: OpSet["source"];
    system: OpSet["system"];
    path: OpSet["path"];
    value: OpSet["value"];
    from?: OpSet["from"];
  }): ValidateResult {
    const op: OpSet = {
      op: OpType.Set,
      source: args.source,
      system: args.system,
      path: args.path,
      value: args.value,
    };
    if (args.from) {
      op.from = args.from;
    }
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

  getPropertyForAllSystems<T>({
    path,
  }: {
    path: OpSet["path"];
  }): Record<string, T> | null {
    const result: Record<string, T> = {};
    for (const system of Object.keys(this.properties)) {
      const value = _.get(this.properties[system], path);
      if (!_.isUndefined(value)) {
        result[system] = value;
      }
    }
    if (Object.keys(result).length == 0) {
      return null;
    } else {
      return result;
    }
  }

  unsetForAllSystems({ path }: { path: OpSet["path"] }): void {
    for (const opSet of _.cloneDeep(this.ops)) {
      if (_.isEqual(opSet.path, path) && opSet.source == OpSource.Inferred) {
        this.addOpUnset({
          op: OpType.Unset,
          path,
          source: opSet.source,
          system: opSet.system,
        });
      }
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

  discoverable(): RegistryEntry[] {
    const discoverable: RegistryEntry[] = [];
    for (const entry of Object.values(registry)) {
      if (entry.discoverableFrom) {
        const match = _.find(
          entry.discoverableFrom,
          (e) => e == this.entityType,
        );
        if (match) {
          if (entry) {
            discoverable.push(entry);
          } else {
            console.log(
              "A discoverable entity was found, but it was not in the registry! bug!",
              { match, entity: this },
            );
          }
        }
      }
    }
    return discoverable;
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

  setDefaultProperties(): void {
    for (const field of this.editFields()) {
      if (
        field.schema.type == "string" ||
        field.schema.type == "number" ||
        field.schema.type == "boolean"
      ) {
        if (field.schema.defaultValue) {
          const op = this.valueOpForPath({
            path: field.path,
            system: "baseline",
          });
          if (!op) {
            this.addOpSet({
              op: OpType.Set,
              path: field.path,
              value: field.schema.defaultValue,
              source: OpSource.Inferred,
              system: "baseline",
            });
          }
        }
      }
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
  valueFromOp({
    path,
    system,
    source,
  }: {
    path: OpSet["path"];
    system: OpSet["system"];
    source: OpSet["source"];
  }): OpSet | undefined {
    const op = _.find(
      this.ops,
      (o) =>
        o.source == source && _.isEqual(o.path, path) && o.system == system,
    );
    if (op) {
      return op as OpSet;
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
