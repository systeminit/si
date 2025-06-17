import { computed, ComputedRef, inject, unref } from "vue";
import { SchemaId, SchemaVariantId } from "@/api/sdf/dal/schema";
import { assertIsDefined, Context } from "../types";

export const useUpgrade = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined<Context>(ctx);

  return (
    schemaId: ComputedRef<SchemaId> | SchemaId,
    schemaVariantId: ComputedRef<SchemaVariantId> | SchemaVariantId,
  ) => {
    return computed(() => {
      const members = ctx.schemaMembers.value[unref(schemaId)];
      if (!members) return false;
      if (
        members.editingVariantId &&
        unref(schemaVariantId) !== members.editingVariantId
      ) {
        return true;
      } else if (
        !members.editingVariantId &&
        unref(schemaVariantId) !== members.defaultVariantId
      ) {
        return true;
      }
      return false;
    });
  };
};
