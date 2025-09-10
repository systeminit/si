// This is where re-usable code for use in actual test execution goes.
import { computed, ComputedRef, MaybeRefOrGetter, toValue } from "vue";
import { EntityKind } from "@/workers/types/entity_kind_types";
import { Gettable } from "@/workers/types/dbinterface";
import { Context } from "@/newhotness/types";

export const innerUseMakeKey = (ctx: Context) => {
  return <K = Gettable>(
    kind: MaybeRefOrGetter<K>,
    id?: MaybeRefOrGetter<string>,
    extension?: MaybeRefOrGetter<string>,
  ) =>
    computed<
      | [string, string, ComputedRef<K> | K, string, string]
      | [string, string, ComputedRef<K> | K, string]
    >(() =>
      extension
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
          ],
    );
};

export const innerUseMakeArgs = (ctx: Context) => {
  return <K = Gettable>(kind: EntityKind, id?: string) => {
    return {
      workspaceId: ctx.workspacePk.value,
      changeSetId: ctx.changeSetId.value,
      kind: kind as K,
      id: id ?? ctx.workspacePk.value,
    };
  };
};
