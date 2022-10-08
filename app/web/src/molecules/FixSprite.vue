<template>
  <SiCollapsible
    as="li"
    class="w-full"
    content-as="ul"
    :default-open="false"
    hide-bottom-border
  >
    <template #label>
      <div
        class="flex flex-row items-center gap-2.5 text-sm relative"
        :class="classes"
      >
        <VormInput
          v-if="fix.status === 'unstarted'"
          :model-value="selected"
          type="checkbox"
          no-label
          @click.stop
          @update:model-value="
            (c) => {
              emit('toggle', c);
            }
          "
        />
        <Icon
          v-else-if="fix.status === 'running'"
          name="loader"
          class="text-action-300"
          size="xl"
        />
        <Icon
          v-else-if="fix.status === 'failure'"
          name="x-circle"
          class="text-destructive-500"
          size="xl"
        />
        <Icon
          v-else-if="fix.status === 'success'"
          name="check-circle"
          class="text-success-500"
          size="xl"
        />
        <Icon
          v-if="fix.status !== 'success'"
          name="tools"
          size="xl"
          class="text-destructive-500"
        />
        <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
          {{ fix.name }}
        </div>
      </div>
    </template>
    <template #default>
      <div class="pl-8 pr-2 py-4">
        <span class="font-bold">Recommendation: </span>{{ fix.recommendation }}
      </div>
    </template>
  </SiCollapsible>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import Icon from "@/ui-lib/Icon.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { Fix } from "@/store/fixes.store";

const props = defineProps({
  fix: { type: Object as PropType<Fix>, required: true },
  class: { type: String },
  selected: { type: Boolean, default: false },
});

const classes = computed(() => props.class);

const emit = defineEmits<{
  (e: "toggle", checked: boolean): void;
}>();
</script>
