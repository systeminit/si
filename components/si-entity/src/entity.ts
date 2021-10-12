import { OpSet, OpSource } from "./ops";
import { SiStorable } from "./siStorable";
import { Tombstone } from "./tombstone";
import _ from "lodash";
import { findProp, ItemProp, Prop } from "si-registry";
import { InvalidContainerPropError, InvalidOpPathError } from "./errors";

export interface IEntity {
  id: string;
  entityType: string;
  nodeId: string;
  name: string;
  siStorable: SiStorable;
  dependencies: Record<string, string[]>;
  ops: Array<OpSet>;
  tombstones: Tombstone[];
  properties: {
    [systemId: string]: Record<string, unknown>;
  };
  code: {
    [systemId: string]: string;
  };
}

export interface FindOpForPathAndSystemArgs {
  path: OpSet["path"];
  system: OpSet["system"];
}

function findOpSetForPathAndSystem(
  entity: IEntity,
  { path, system }: FindOpForPathAndSystemArgs,
): OpSet | undefined {
  return _.find(
    entity.ops,
    (o) =>
      _.isEqual(o.path, path) && (o.system == system || o.system == "baseline"),
  );
}

export function emptyValueForContainerProp(
  prop: ReturnType<typeof findProp>,
): Record<string, unknown> | [] {
  if (prop.type == "object") {
    return {};
  } else if (prop.type == "array") {
    return [];
  } else if (prop.type == "map") {
    return {};
  } else {
    throw new InvalidContainerPropError({ prop });
  }
}

export function updateOpSet(entity: IEntity, op: OpSet): void {
  const existingIndex = _.findIndex(entity.ops, (o) =>
    OpSet.isEqualExceptValue(o, op),
  );
  // -1 means there is no op that matches
  if (existingIndex == -1) {
    entity.ops.push(op);
  } else {
    removeDependencies(entity, entity.ops[existingIndex].id);
    entity.ops[existingIndex] = op;
  }
}

export function removeDependencies(entity: IEntity, opSetId: string): IEntity {
  delete entity.dependencies[opSetId];
  return entity;
}

export interface UpdateDependenciesArgs {
  opSetId: string;
  dependencies: string[];
}

export function updateDependencies(
  entity: IEntity,
  { opSetId, dependencies }: UpdateDependenciesArgs,
): IEntity {
  entity.dependencies[opSetId] = dependencies;
  return entity;
}

function listSystemsForEntity(entity: IEntity): string[] {
  const systems: Set<string> = new Set();
  systems.add("baseline");
  for (const opSet of entity.ops) {
    systems.add(opSet.system);
  }
  return Array.from(systems);
}

export interface IsOpSetOverridenArgs {
  opSet: {
    path: OpSet["path"];
    source: OpSet["source"];
    system: OpSet["system"];
    value: OpSet["value"];
  };
  system: string;
}

export function isOpSetOverridden(
  entity: IEntity,
  { opSet, system }: IsOpSetOverridenArgs,
): boolean {
  if (opSet.source == OpSource.Inferred && opSet.system == "baseline") {
    const override = _.find(entity.ops, (toCheckOp) => {
      // The item we are checking is never overriden.
      if (OpSet.isEqual(toCheckOp, opSet)) {
        return false;
      }

      return (
        _.isEqual(toCheckOp.path, opSet.path) &&
        ((toCheckOp.source == OpSource.Manual &&
          toCheckOp.system == "baseline") ||
          toCheckOp.system == system) &&
        !isTombstoned(entity, toCheckOp)
      );
    });
    if (override) {
      return true;
    }
  } else if (opSet.source == OpSource.Inferred && opSet.system != "baseline") {
    const override = _.find(entity.ops, (p) => {
      // Our own object is not an override!
      if (OpSet.isEqual(p, opSet)) {
        return false;
      }

      return (
        _.isEqual(p.path, opSet.path) &&
        p.system == opSet.system &&
        opSet.system == system &&
        !isTombstoned(entity, p)
      );
    });
    if (override) {
      return true;
    }
  } else if (opSet.source == OpSource.Manual && opSet.system == "baseline") {
    const override = _.find(entity.ops, (p) => {
      if (OpSet.isEqual(p, opSet)) {
        return false;
      }
      return (
        _.isEqual(p.path, opSet.path) &&
        p.source == OpSource.Manual &&
        p.system == system &&
        !isTombstoned(entity, p)
      );
    });
    if (override) {
      return true;
    }
  }
  return false;
}

function subPath(long: string[], short: string[]): boolean {
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

export interface IsOpSetTombstonedArgs {
  path: OpSet["path"];
  source: OpSet["source"];
  system: OpSet["system"];
}

export function removeTombstone(
  entity: IEntity,
  tombstone: Tombstone,
): IEntity {
  _.remove(entity.tombstones, (t) => _.isEqual(t, tombstone));
  return entity;
}

export function addTombstone(entity: IEntity, tombstone: Tombstone): IEntity {
  const existingTombstone = _.find(entity.tombstones, (t) =>
    _.isEqual(t, tombstone),
  );
  if (!existingTombstone) {
    entity.tombstones.push(tombstone);
  }
  return entity;
}

export function isTombstoned(
  entity: IEntity,
  { path, source, system }: IsOpSetTombstonedArgs,
): boolean {
  for (const tombstone of entity.tombstones) {
    if (
      subPath(path, tombstone.path) &&
      source == tombstone.source &&
      system == tombstone.system
    ) {
      return true;
    }
  }
  return false;
}

export function generateProperties(entity: IEntity): IEntity {
  //const newProperties: Record<string, unknown> = {};
  //const systems = listSystemsForEntity(entity);
  //for (const system of systems) {
  //  entity.properties[system] = {};
  //}
  //for (const opSet of entity.ops) {
  //  if (opSet.system == "baseline") {
  //    for (const system of systems) {
  //      if (!this.isOverridden(op, system) && !this.isTombstoned(op)) {
  //        _.set(newProperties[system], op.path, _.cloneDeep(op.value));
  //      }
  //    }
  //  }
  //}
  return entity;
}

export interface EntitySetValueArgs {
  path: OpSet["path"];
  system: OpSet["system"];
  value: OpSet["value"];
  editPartial?: OpSet["editPartial"];
  provenance?: OpSet["provenance"];
}

function setValue(
  entity: IEntity,
  { path, system, value, editPartial, provenance }: EntitySetValueArgs,
): IEntity {
  // Walk the path, creating interstitial object as needed, essentially
  // duplicating the auto-viv of lodash

  const dependencies = [];

  if (path.length >= 2) {
    for (let x = 0; x < path.length - 1; x++) {
      const currentPath = path.slice(0, x + 1);
      const existingOp = findOpSetForPathAndSystem(entity, {
        path: currentPath,
        system,
      });
      if (existingOp) {
        dependencies.push(existingOp.id);
      } else {
        const prop = findProp([entity.entityType, ...currentPath]);
        if (!prop) {
          throw new InvalidOpPathError({
            entityType: entity.entityType,
            path: currentPath,
          });
        }
        const emptyValue = emptyValueForContainerProp(prop);
        let dependentOp;
        if (provenance) {
          dependentOp = OpSet.createInferred({
            path: currentPath,
            system,
            value: emptyValue,
            entityType: entity.entityType,
            provenance,
          });
        } else {
          dependentOp = OpSet.createManual({
            path: currentPath,
            system,
            value: emptyValue,
            entityType: entity.entityType,
          });
        }
        updateOpSet(entity, dependentOp);
        updateDependencies(entity, { opSetId: dependentOp.id, dependencies });
      }
    }
  }

  let opSet;
  if (provenance) {
    opSet = OpSet.createInferred({
      path,
      system,
      value,
      editPartial,
      entityType: entity.entityType,
      provenance,
    });
  } else {
    opSet = OpSet.createManual({
      path,
      system,
      value,
      editPartial,
      entityType: entity.entityType,
    });
  }
  updateOpSet(entity, opSet);
  updateDependencies(entity, { opSetId: opSet.id, dependencies });

  return entity;
}

export interface EntitySetManualArgs {
  path: OpSet["path"];
  system: OpSet["system"];
  value: OpSet["value"];
  editPartial?: OpSet["editPartial"];
}

export interface EntitySetInferredArgs {
  path: OpSet["path"];
  system: OpSet["system"];
  value: OpSet["value"];
  editPartial?: OpSet["editPartial"];
  provenance: OpSet["provenance"];
}

function setManualValue(entity: IEntity, args: EntitySetManualArgs): IEntity {
  return setValue(entity, args);
}

function setInferredValue(
  entity: IEntity,
  args: EntitySetInferredArgs,
): IEntity {
  return setValue(entity, args);
}

export const Entity = {
  setManualValue,
  setInferredValue,
  addTombstone,
  removeTombstone,
  findOpSetForPathAndSystem,
};
