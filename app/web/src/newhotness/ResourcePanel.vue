<template>
  <ul class="p-xs">
    <pre>
      {{ JSON.stringify(resource, null, 2) }}
    </pre>
  </ul>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import {
  BifrostAttributeTree,
  BifrostComponent,
} from "@/workers/types/dbinterface";

const props = defineProps<{
  attributeValueId: string;
  component: BifrostComponent;
}>();

const attributeTreeMakeKey = makeKey("AttributeTree", props.attributeValueId);
const attributeTreeMakeArgs = makeArgs("AttributeTree", props.attributeValueId);
const attributeTreeQuery = useQuery<BifrostAttributeTree | null>({
  queryKey: attributeTreeMakeKey,
  queryFn: async () =>
    await bifrost<BifrostAttributeTree>(attributeTreeMakeArgs),
});

const root = computed(() => attributeTreeQuery.data.value);

const resource = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "resource"),
);
</script>
