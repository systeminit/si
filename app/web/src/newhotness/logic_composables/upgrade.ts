import { inject } from "vue";
import { SchemaId, SchemaVariantId } from "@/api/sdf/dal/schema";
import { assertIsDefined, Context } from "../types";

export const useUpgrade = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined<Context>(ctx);

  return (schemaId: SchemaId, schemaVariantId: SchemaVariantId) => {
    const members = ctx.schemaMembers.value[schemaId];
    if (!members) return false;
    if (
      members.editingVariantId &&
      schemaVariantId !== members.editingVariantId
    ) {
      return true;
    } else if (
      !members.editingVariantId &&
      schemaVariantId !== members.defaultVariantId
    ) {
      return true;
    }
    return false;
  };
};
