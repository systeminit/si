<template>
  <ul class="p-xs">
    <CodeViewer
      v-if="attributeValueId && resourcePayload"
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
import { BifrostAttributeTree } from "@/workers/types/dbinterface";
import CodeViewer from "@/components/CodeViewer.vue";
import EmptyStateCard from "@/components/EmptyStateCard.vue";

const props = defineProps<{
  attributeValueId?: string;
}>();

const attributeValueId = computed(() => props.attributeValueId ?? "");

const key = useMakeKey();
const args = useMakeArgs();
const attributeTreeQuery = useQuery<BifrostAttributeTree | null>({
  queryKey: key("AttributeTree", attributeValueId),
  enabled: !!attributeValueId.value,
  queryFn: async () =>
    await bifrost<BifrostAttributeTree>(
      args("AttributeTree", attributeValueId.value),
    ),
});

const root = computed(() => attributeTreeQuery.data.value);

const resourcePayload = computed(() => {
  const resourceTree = root.value?.children.find(
    (child) => child.prop?.name === "resource",
  );
  const payloadAttr = resourceTree?.children.find(
    (child) => child.prop?.name === "payload",
  );
  const payload = payloadAttr?.attributeValue.value;
  if (!payload) return undefined;
  return payload;
});
</script>
