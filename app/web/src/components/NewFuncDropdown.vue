<template>
  <VButton2
    tone="action"
    icon="plus"
    icon-right="chevron--down"
    :variant="menuRef?.isOpen ? 'ghost' : 'solid'"
    :request-status="requestStatus"
    loading-text="Creating new function..."
    size="sm"
    @click="menuRef?.open"
  >
    {{ label }}
    <DropdownMenu ref="menuRef">
      <DropdownMenuItem
        v-for="(fnLabel, fnVariant) in fnTypes"
        :key="fnVariant"
        @select="emit('selectedFuncVariant', fnVariant)"
      >
        <template #icon><FuncSkeleton /></template>
        {{ fnLabel }}
      </DropdownMenuItem>
    </DropdownMenu>
  </VButton2>
</template>

<script setup lang="ts">
import { PropType, ref } from "vue";
import FuncSkeleton from "@/components/FuncSkeleton.vue";
import { FuncVariant } from "@/api/sdf/dal/func";
import DropdownMenu from "@/ui-lib/menus/DropdownMenu.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import DropdownMenuItem from "@/ui-lib/menus/DropdownMenuItem.vue";
import { ApiRequestStatus } from "@/store/lib/pinia_api_tools";

const props = defineProps({
  label: { type: String, required: true },
  fnTypes: {
    type: Object as PropType<{ [key in FuncVariant]?: string }>,
    required: true,
  },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});

const emit = defineEmits<{
  (e: "selectedFuncVariant", kind: FuncVariant): void;
}>();

const menuRef = ref<InstanceType<typeof DropdownMenu>>();
</script>
