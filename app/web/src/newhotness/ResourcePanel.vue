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
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostAttributeTree,
  BifrostComponent,
} from "@/workers/types/dbinterface";

const props = defineProps<{
  attributeValueId: string;
  component: BifrostComponent;
}>();

const attributeValueId = computed(() => props.attributeValueId);

const key = useMakeKey();
const args = useMakeArgs();
const attributeTreeQuery = useQuery<BifrostAttributeTree | null>({
  queryKey: key("AttributeTree", attributeValueId),
  queryFn: async () =>
    await bifrost<BifrostAttributeTree>(
      args("AttributeTree", attributeValueId.value),
    ),
});

const root = computed(() => attributeTreeQuery.data.value);

const resource = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "resource"),
);
</script>
