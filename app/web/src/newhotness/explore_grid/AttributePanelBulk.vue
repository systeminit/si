<template>
  <div class="scrollable grow">
    <ul class="flex flex-row gap-xs">
      <li v-for="component in selectedComponents" :key="component.id">
        <ComponentCard :component="component"/>
      </li>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { bifrost, useMakeArgs, useMakeKey } from '@/store/realtime/heimdall';
import { AttributeTree, ComponentInList, EntityKind } from '@/workers/types/entity_kind_types';
import { useQueries } from '@tanstack/vue-query';
import { computed, onBeforeUnmount, onMounted } from 'vue';
import { keyEmitter } from '@/newhotness/logic_composables/emitters';
import ComponentCard from "@/newhotness/ComponentCard.vue";

const props = defineProps<{
  selectedComponents: ComponentInList[],
}>();

const componentIds = computed(() => props.selectedComponents.map(c => c.id))

const makeKey = useMakeKey();
const makeArgs = useMakeArgs();

const queries = computed(() => componentIds.value.map((id) => {
  return {
    queryKey: makeKey(EntityKind.AttributeTree, id),
    queryFn: async () => (await bifrost<AttributeTree>(
      makeArgs(EntityKind.AttributeTree, id))
    )
  }
}))
const avTrees = useQueries({
  queries
})

const onEscape = () => {
  emit("close");
}
onMounted(() => {
  keyEmitter.on("Escape", onEscape);
});
onBeforeUnmount(() => {
  keyEmitter.on("Escape", onEscape);
});

const emit = defineEmits<{
  (e: "close"): void;
}>()
</script>