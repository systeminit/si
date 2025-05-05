<template>
  <ul class="p-xs">
    <li v-for="item in items" :key="item.name">
      <TextPill>{{ item.result }}</TextPill> {{ item.name }}: {{ item.message }}
    </li>
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
import TextPill from "./layout_components/TextPill.vue";

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

const quals = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "qualification"),
);

interface QualItem {
  name: string;
  message: string;
  result: string;
}

const items = computed<QualItem[]>(() => {
  return (
    quals.value?.children.map((c): QualItem => {
      const name = c.attributeValue.key ?? "";
      const message =
        c.children.find((_c) => _c.prop?.name === "message")?.attributeValue
          .value ?? "";
      const result =
        c.children.find((_c) => _c.prop?.name === "result")?.attributeValue
          .value ?? "";
      return {
        name,
        message,
        result,
      };
    }) ?? []
  );
});
</script>
