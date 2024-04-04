<template>
  <VButton
    tone="action"
    icon="plus"
    iconRight="chevron--down"
    :variant="menuRef?.isOpen ? 'ghost' : 'solid'"
    :requestStatus="requestStatus"
    loadingText="Creating new function..."
    size="sm"
    @click="menuRef?.open"
  >
    {{ label }}
    <DropdownMenu ref="menuRef">
      <DropdownMenuItem
        v-for="(fnLabel, fnKind) in CUSTOMIZABLE_FUNC_TYPES"
        :key="fnKind"
        @select="emit('selectedFuncKind', fnKind)"
      >
        <template #icon><FuncSkeleton /></template>
        {{ fnLabel.singularLabel }}
      </DropdownMenuItem>
    </DropdownMenu>
  </VButton>
</template>

<script setup lang="ts">
import { PropType, ref } from "vue";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import {
  DropdownMenu,
  DropdownMenuItem,
  VButton,
} from "@si/vue-lib/design-system";
import FuncSkeleton from "@/components/FuncSkeleton.vue";
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
} from "@/api/sdf/dal/func";

defineProps({
  label: { type: String, required: true },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});

const emit = defineEmits<{
  (e: "selectedFuncKind", kind: CustomizableFuncKind): void;
}>();

const menuRef = ref<InstanceType<typeof DropdownMenu>>();
</script>
