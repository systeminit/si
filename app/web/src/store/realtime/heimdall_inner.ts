// This is where re-usable code for use in actual test execution goes.
import { computed, ComputedRef, MaybeRefOrGetter, toValue } from "vue";
import {
  EntityKind,
  GLOBAL_ENTITIES,
  GLOBAL_IDENTIFIER,
  GlobalEntity,
} from "@/workers/types/entity_kind_types";
import { Gettable } from "@/workers/types/dbinterface";
import { Context } from "@/newhotness/types";

export const rawUseMakeKey = (
  ctx: Pick<Context, "workspacePk" | "changeSetId">,
) => {
  return <K = Gettable>(
    kind: MaybeRefOrGetter<K>,
    id?: MaybeRefOrGetter<string>,
    extension?: MaybeRefOrGetter<string>,
  ) =>
    computed<
      | [string, string, ComputedRef<K> | K, string, string]
      | [string, string, ComputedRef<K> | K, string]
      | [string, ComputedRef<K> | K, string]
    >(() => {
      if (GLOBAL_ENTITIES.includes(kind as GlobalEntity)) {
        return [
          GLOBAL_IDENTIFIER,
          toValue(kind),
          toValue(id) ?? GLOBAL_IDENTIFIER,
        ];
      } else
        return extension
          ? [
              ctx.workspacePk.value,
              ctx.changeSetId.value,
              toValue(kind),
              toValue(id ?? ctx.workspacePk.value),
              toValue(extension),
            ]
          : [
              ctx.workspacePk.value,
              ctx.changeSetId.value,
              toValue(kind),
              toValue(id ?? ctx.workspacePk),
            ];
    });
};

export const innerUseMakeKey = (ctx: Context) => {
  return rawUseMakeKey(ctx);
};

export const rawUseMakeArgs = (
  ctx: Pick<Context, "workspacePk" | "changeSetId">,
) => {
  return <K = Gettable | GlobalEntity>(kind: EntityKind, id?: string) => {
    if (GLOBAL_ENTITIES.includes(kind as GlobalEntity)) {
      return {
        workspaceId: "-",
        changeSetId: "-",
        kind: kind as K,
        id: id ?? "-",
      };
    }
    return {
      workspaceId: ctx.workspacePk.value,
      changeSetId: ctx.changeSetId.value,
      kind: kind as K,
      id: id ?? ctx.workspacePk.value,
    };
  };
};

export const innerUseMakeArgs = (ctx: Context) => {
  return rawUseMakeArgs(ctx);
};
