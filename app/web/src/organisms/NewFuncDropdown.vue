<template>
  <VButton2
    tone="action"
    icon="plus"
    icon-right="chevron--down"
    :variant="menuRef?.isOpen ? 'ghost' : 'solid'"
    size="sm"
    @click="menuRef?.open"
  >
    {{ label }}
  </VButton2>
  <DropdownMenu ref="menuRef">
    <DropdownMenuItem
      v-for="(fnLabel, fnKind) in fnTypes"
      :key="fnKind"
      @select="emit('selectedFuncKind', fnKind)"
    >
      <template #icon><FuncSkeleton /></template>
      {{ fnLabel }}
    </DropdownMenuItem>
  </DropdownMenu>
</template>

<script setup lang="ts">
import { PropType, ref } from "vue";
import FuncSkeleton from "@/atoms/FuncSkeleton.vue";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import DropdownMenu from "@/ui-lib/menus/DropdownMenu.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import DropdownMenuItem from "@/ui-lib/menus/DropdownMenuItem.vue";

const props = defineProps({
  label: { type: String, required: true },
  fnTypes: {
    type: Object as PropType<{ [key in FuncBackendKind]?: string }>,
    required: true,
  },
});

const emit = defineEmits<{
  (e: "selectedFuncKind", kind: FuncBackendKind): void;
}>();

const menuRef = ref<InstanceType<typeof DropdownMenu>>();
</script>
