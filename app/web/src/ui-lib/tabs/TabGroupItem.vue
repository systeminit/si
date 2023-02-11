<template>
  <div v-if="false">
    <!-- all rendering is done by the TabGroup parent -->
    <!-- default slot content will not work here -->
    <slot name="label" />
    <slot name="stickyTop" />
    <slot />
    <slot name="stickyBottom" />
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted, onBeforeUnmount, getCurrentInstance } from "vue";
import { useTabGroupContext } from "./TabGroup.vue";
import type { Slot } from "vue";

export type TabGroupItemDefinition = {
  props: { slug: string; label: string };
  slots: {
    default?: Slot;
    label?: Slot;
    stickyTop?: Slot;
    stickyBottom?: Slot;
  };
};

const props = defineProps({
  label: { type: String },
  slug: { type: String, default: () => `tab-group-${idCounter++}` },
});

const emit = defineEmits<{ (e: "select"): void }>();

const menuCtx = useTabGroupContext();

const labelText = ref();
const labelRef = ref<HTMLElement>();

onMounted(() => {
  // track text in label to be used for typing to jump to an option
  labelText.value = labelRef.value?.textContent?.toLowerCase().trim();

  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  menuCtx.registerTab(
    props.slug,
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    getCurrentInstance()! as unknown as TabGroupItemDefinition,
  );
});
onBeforeUnmount(() => {
  menuCtx.unregisterTab(props.slug);
});
</script>

<script lang="ts">
let idCounter = 1;
</script>
