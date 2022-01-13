<template>
  <section>
    <div class="flex w-full pt-1 pb-1 mt-2 text-sm text-white">
      <div v-if="isOpen" class="flex">
        <VueFeather type="chevron-down" />
        {{ editField.name }}
      </div>

      <div v-else-if="!isOpen" class="flex">
        <VueFeather type="chevron-right" />
        {{ editField.name }}
      </div>
    </div>
    <div v-show="isOpen" class="flex w-full pt-1 pb-1 mt-2 text-sm text-white">
      <Widgets :edit-fields="widget.options.edit_fields" />
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, PropType, ref } from "vue";
import type { EditField, HeaderWidgetDal } from "@/api/sdf/dal/edit_field";
import VueFeather from "vue-feather";
import Widgets from "@/organisims/EditForm/Widgets.vue";

const props = defineProps({
  show: {
    type: Boolean,
    required: true,
  },
  editField: {
    type: Object as PropType<EditField>,
    required: true,
  },
});

const widget = computed<HeaderWidgetDal>(() => {
  console.log({ widget: JSON.stringify(props.editField.widget) });
  return props.editField.widget as HeaderWidgetDal;
});

const isOpen = ref(true);
</script>

<style scoped>
.property-section-title-bg-color {
  background-color: #292c2d;
}

.section-content {
  @apply overflow-hidden transition duration-150 ease-in-out;
}

.is-closed .section-content {
  @apply overflow-hidden h-0;
}
</style>
