import { OpSource, OpType, SiEntity } from "si-entity";
import { InferPropertiesRequest } from "../controllers/inferProperties";
import _ from "lodash";
import Debug from "debug";
import { findProp } from "si-registry";
const debug = Debug("veritech:controllers:intel:inferShared");

export type Context = InferPropertiesRequest["context"];

export function allEntitiesByType(
  context: Context,
  entityType: string,
): SiEntity[] {
  const entities = _.filter(
    context,
    (entity) => entity.entityType == entityType,
  );
  return entities;
}

export interface SetArrayEntryFromAllEntities {
  entity: SiEntity;
  context: Context;
  entityType: string;
  toPath: string[];
  valuesCallback: (
    fromEntity: SiEntity,
  ) => { path: string[]; value: any; system: string }[];
}

function setEntityFromValuesToSet(
  entity: SiEntity,
  matchingEntity: SiEntity,
  pathRoot: string[],
  valuesToSet: ReturnType<SetArrayEntryFromAllEntities["valuesCallback"]>,
) {
  for (const valueToSet of valuesToSet) {
    const newPath = _.concat(pathRoot, valueToSet.path);
    const fullPath = [entity.entityType].concat(newPath);
    const schema = findProp(fullPath);
    if (schema.type == "array") {
      entity.addOpUnset({
        op: OpType.Unset,
        source: OpSource.Inferred,
        path: newPath,
        system: "baseline",
      });

      // Get the new index for the array.
      const arrayMetaKey = entity.pathToString(fullPath);
      let index = entity.arrayMeta[arrayMetaKey]?.length;
      if (!index) {
        index = 0;
      }
      const fullNewPathRoot = fullPath.concat(`${index}`);
      const newPathRoot = newPath.concat(`${index}`);

      const schema = findProp(fullNewPathRoot);
      let initialValue;
      if (schema.type == "array") {
        if (schema.itemProperty.type == "string") {
          initialValue = "";
        } else if (schema.itemProperty.type == "number") {
          initialValue = 0;
        } else if (schema.itemProperty.type == "boolean") {
          initialValue = false;
        } else if (schema.itemProperty.type == "object") {
          initialValue = {};
        } else if (schema.itemProperty.type == "array") {
          initialValue = [];
        } else if (schema.itemProperty.type == "map") {
          initialValue = {};
        }
      }
      entity.set({
        source: OpSource.Inferred,
        path: newPathRoot,
        // @ts-ignore
        value: initialValue,
        system: "baseline",
        from: {
          entityId: matchingEntity.id,
          entityType: matchingEntity.entityType,
          arrayRoot: true,
        },
      });
      entity.computeProperties();
      const nextValueSet: ReturnType<
        SetArrayEntryFromAllEntities["valuesCallback"]
      > = [];
      if (_.isObjectLike(valueToSet.value)) {
        for (const key in valueToSet.value) {
          const path = [key];
          nextValueSet.push({
            path,
            value: valueToSet.value[key],
            system: valueToSet.system,
          });
        }
      } else if (_.isArray(valueToSet.value)) {
        for (let x = 0; x < valueToSet.value.length; x++) {
          const path = [`${x}`];
          nextValueSet.push({
            path,
            value: valueToSet.value[x],
            system: valueToSet.system,
          });
        }
      } else {
        const path = ["0"];
        nextValueSet.push({
          path,
          value: valueToSet.value,
          system: valueToSet.system,
        });
      }
      setEntityFromValuesToSet(
        entity,
        matchingEntity,
        newPathRoot,
        nextValueSet,
      );
    } else {
      entity.set({
        source: OpSource.Inferred,
        path: newPath,
        system: valueToSet.system,
        value: valueToSet.value,
        from: {
          entityId: matchingEntity.id,
          entityType: matchingEntity.entityType,
        },
      });
      entity.computeProperties();
    }
  }
}

export function setArrayEntryFromAllEntities(
  args: SetArrayEntryFromAllEntities,
): SiEntity {
  const startingOps = _.cloneDeep(args.entity.ops);

  // Remove all previously inferred path and from entityType.
  for (const op of startingOps) {
    if (
      op.source == OpSource.Inferred &&
      op.from?.entityType == args.entityType
    ) {
      if (!op.from?.arrayRoot) {
        args.entity.addOpUnset({
          op: OpType.Unset,
          system: op.system,
          path: op.path,
          source: OpSource.Inferred,
        });
      }
    }
  }

  // Set the values for each matching entity
  const matchingEntities = allEntitiesByType(args.context, args.entityType);
  for (const matchingEntity of matchingEntities) {
    const valuesToSet = args.valuesCallback(matchingEntity);
    const arrayRoot = _.find(args.entity.ops, (o) => {
      return (
        o.source == OpSource.Inferred &&
        o.from?.entityId == matchingEntity.id &&
        o.from?.arrayRoot &&
        matchingEntity.subPath(o.path, args.toPath)
      );
    });
    if (arrayRoot) {
      setEntityFromValuesToSet(
        args.entity,
        matchingEntity,
        arrayRoot.path,
        valuesToSet,
      );
    } else {
      // Add a new entry!
      const fullPath = [args.entity.entityType].concat(args.toPath);
      const arrayMetaKey = args.entity.pathToString(fullPath);
      let index = args.entity.arrayMeta[arrayMetaKey]?.length;
      if (!index) {
        index = 0;
      }
      const fullNewPathRoot = fullPath.concat(`${index}`);
      const newPathRoot = args.toPath.concat(`${index}`);
      const schema = findProp(fullNewPathRoot);
      let initialValue;
      debug("about to fail", { schema, fullPath });
      if (schema.type == "array") {
        if (schema.itemProperty.type == "string") {
          initialValue = "";
        } else if (schema.itemProperty.type == "number") {
          initialValue = 0;
        } else if (schema.itemProperty.type == "boolean") {
          initialValue = false;
        } else if (schema.itemProperty.type == "object") {
          initialValue = {};
        } else if (schema.itemProperty.type == "array") {
          initialValue = [];
        } else if (schema.itemProperty.type == "map") {
          initialValue = {};
        }
      }
      args.entity.set({
        source: OpSource.Inferred,
        path: newPathRoot,
        // @ts-ignore
        value: initialValue,
        system: "baseline",
        from: {
          entityId: matchingEntity.id,
          entityType: matchingEntity.entityType,
          arrayRoot: true,
        },
      });
      args.entity.computeProperties();
      setEntityFromValuesToSet(
        args.entity,
        matchingEntity,
        newPathRoot,
        valuesToSet,
      );
    }
  }

  // Prune all roots that have no values set at all.
  const pruneOps = _.cloneDeep(args.entity.ops);
  for (const op of pruneOps) {
    if (
      op.source == OpSource.Inferred &&
      op.from?.entityType == args.entityType &&
      op.from?.arrayRoot
    ) {
      const hasItem = _.find(args.entity.ops, (o) => {
        return args.entity.subPath(o.path, op.path);
      });
      if (!hasItem) {
        args.entity.addOpUnset({
          op: OpType.Unset,
          path: op.path,
          source: op.source,
          system: op.system,
        });
      }
    }
  }
  return args.entity;
}

export function findEntityByType(
  context: Context,
  entityType: string,
): SiEntity | null {
  const entity = _.find(context, (entity) => entity.entityType == entityType);
  if (entity) {
    return entity;
  } else {
    return null;
  }
}

interface FindPropertyResult<T> {
  entity: SiEntity;
  properties: Record<string, T>;
}

export function findProperty<T>(
  context: Context,
  entityType: string,
  path: string[],
): FindPropertyResult<T> | null {
  const entity = findEntityByType(context, entityType);
  if (entity) {
    const properties: Record<string, T> = entity.getPropertyForAllSystems({
      path,
    });
    if (properties) {
      return { entity, properties };
    } else {
      return null;
    }
  } else {
    return null;
  }
}

export interface SetPropertyFromProperty {
  entity: SiEntity;
  toPath: string[];
  fromPath: string[];
  system?: string;
}

export function setPropertyFromProperty({
  entity,
  toPath,
  fromPath,
}: SetPropertyFromProperty): SiEntity {
  setPropertyFromEntity({
    context: [entity],
    entityType: entity.entityType,
    fromPath,
    toEntity: entity,
    toPath,
  });
  return entity;
}

export interface SetProperty {
  entity: SiEntity;
  toPath: string[];
  value: any;
  system?: string;
}

export function setProperty({
  entity,
  toPath,
  value,
  system,
}: SetProperty): SiEntity {
  if (!system) {
    system = "baseline";
  }
  entity.set({
    source: OpSource.Inferred,
    path: toPath,
    value,
    system,
  });
  entity.computeProperties();
  return entity;
}

export interface SetPropertyFromEntityArgs {
  context: Context;
  entityType: string;
  fromPath: string[];
  toEntity: SiEntity;
  toPath: string[];
}

export function setPropertyFromEntity({
  context,
  entityType,
  fromPath,
  toEntity,
  toPath,
}: SetPropertyFromEntityArgs): SiEntity {
  toEntity.unsetForAllSystems({ path: toPath });
  const newValue = findProperty<string>(context, entityType, fromPath);
  if (newValue) {
    for (const system in newValue.properties) {
      debug("----- set property from value");
      debug({
        value: newValue.properties[system],
        system,
        toPath,
      });
      toEntity.set({
        source: OpSource.Inferred,
        system,
        path: toPath,
        value: newValue.properties[system],
        from: {
          entityId: newValue.entity.id,
          entityType: newValue.entity.entityType,
        },
      });
      toEntity.computeProperties();
    }
  }
  return toEntity;
}
