<template>
  <div class="p-md">
    <span class="font-bold">ATTRIBUTE PANEL</span>
    {{ attributeTree }}
  </div>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import { BifrostAttributeTree } from "@/workers/types/dbinterface";

const props = defineProps<{
  attributeValueId: string;
}>();

const attributeTreeMakeKey = makeKey("AttributeTree", props.attributeValueId);
const attributeTreeMakeArgs = makeArgs("AttributeTree", props.attributeValueId);
const attributeTree = useQuery<BifrostAttributeTree | null>({
  queryKey: attributeTreeMakeKey,
  queryFn: async () =>
    await bifrost<BifrostAttributeTree>(attributeTreeMakeArgs),
});
</script>
