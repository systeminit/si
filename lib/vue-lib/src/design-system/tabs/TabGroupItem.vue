<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <!-- tab top content, usually filled using label prop, but slot in case something special is needed-->
  <!-- NOTE - default slot content will not work here, due to how the slot render fn is being called by the parent -->
  <slot v-if="false" name="label" />

  <!-- if tab is selected, teleport default slot into the main tab content area -->
  <template v-if="slug === menuCtx.selectedTabSlug.value">
    <SafeTeleport :to="`#${menuCtx.teleportId}`">
      <slot />
    </SafeTeleport>
  </template>
</template>

<script lang="ts" setup>
import { ref, onMounted, onBeforeUnmount, getCurrentInstance } from "vue";
import { SafeTeleport } from "vue-safe-teleport";
import { useTabGroupContext } from "./TabGroup.vue";
import type { Slot } from "vue";

export type TabGroupItemDefinition = {
  props: { slug: string; label: string; uncloseable: boolean };
  slots: {
    default?: Slot;
    label?: Slot;
    top?: Slot;
    bottom?: Slot;
  };
};

const props = defineProps({
  label: { type: String },
  uncloseable: { type: Boolean, default: false },
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
