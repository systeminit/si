<template>
  <ul class="p-xs">
    <CodeViewer
      v-if="resourcePayload"
      :code="JSON.stringify(resourcePayload, null, 2)"
    />
    <EmptyStateCard
      v-else
      iconName="no-changes"
      primaryText="No Resource"
      secondaryText="This component does not have a resource associated with it."
    />
  </ul>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { AttributeTree } from "@/workers/types/dbinterface";
import CodeViewer from "@/components/CodeViewer.vue";
import EmptyStateCard from "@/components/EmptyStateCard.vue";
import { findAvsAtPropPath } from "./util";

const props = defineProps<{
  componentId?: string;
}>();

const componentId = computed(() => props.componentId ?? "");

const key = useMakeKey();
const args = useMakeArgs();
const attributeTreeQuery = useQuery<AttributeTree | null>({
  queryKey: key("AttributeTree", componentId),
  queryFn: async () =>
    await bifrost<AttributeTree>(args("AttributeTree", componentId.value)),
});

const root = computed(() => attributeTreeQuery.data.value);

const resourcePayload = computed(() => {
  if (!root.value) return;
  const data = findAvsAtPropPath(root.value, ["root", "resource", "payload"]);
  if (!data) return;
  const { attributeValues } = data;
  // only one AV for /resource/payload
  return attributeValues[0]?.value;
});
</script>
