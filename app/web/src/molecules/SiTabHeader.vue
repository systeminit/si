<template>
  <!-- selectedToFront ? (selected ? 'order-1' : 'order-2') : '' -->
  <Tab
    v-slot="{ selected }"
    class="focus:outline-none whitespace-nowrap"
    :style="maxWidthStyle"
    as="template"
  >
    <button
      :class="
        selectedToFront
          ? selected
            ? moveTabToFrontIfOverflowing($el.nextElementSibling)
            : 'order-2'
          : ''
      "
    >
      <span
        class="w-full"
        :class="
          classesF + ' ' + (selected ? selectedClassesF : defaultClassesF)
        "
      >
        <div class="overflow-hidden text-ellipsis">
          <slot />
        </div>
        <div>
          <slot name="icon" />
        </div>
      </span>
    </button>
  </Tab>
  <div
    v-if="afterMargin > 0"
    class="border-b border-neutral-300 dark:border-neutral-600"
    :class="'w-' + afterMargin + (selectedToFront ? ' order-2' : '')"
  ></div>
</template>

<script setup lang="ts">
import { Tab } from "@headlessui/vue";
import { computed, inject } from "vue";

const props = defineProps<{
  classes?: string;
  defaultClasses?: string;
  selectedClasses?: string;
}>();

const afterMargin = inject("afterMargin", 0);
const selectedToFront = inject("selectedTabToFront", false);
const classesF = inject("tabClasses", props.classes);
const defaultClassesF = inject("defaultTabClasses", props.defaultClasses);
const selectedClassesF = inject("selectedTabClasses", props.selectedClasses);
const maximumWidth = inject("tabWidthMaximum", 0);

const maxWidthStyle = computed(() => {
  if (maximumWidth <= 0) return "max-width: 90%"; // By default, tabs cannot take up more than 90% of the tab area
  if (maximumWidth < 1) return `max-width: ${Math.floor(maximumWidth * 100)}%`; // values from 0 to 1 are converted to percentages
  return `max-width: ${maximumWidth}px`;
});

const moveTabToFrontIfOverflowing = (tabElement: HTMLElement) => {
  const parent = tabElement.parentElement;
  if (!parent) return "order-2"; // no parent? don't reorder elements

  const tabInnerAreaWidth = parent.clientWidth;
  const tabWidth = tabElement.getBoundingClientRect().width;
  const allButtons = [...parent.children].filter((element) =>
    element.querySelector("button"),
  );
  let priorTabsWidth = 0;
  for (let i = 0; i < allButtons.length; i++) {
    const priorTabElement = allButtons[i];
    if (priorTabElement === tabElement) i = allButtons.length;
    else priorTabsWidth += priorTabElement.getBoundingClientRect().width;
  }

  if (priorTabsWidth + tabWidth > tabInnerAreaWidth) return "order-1";
  return "order-2";
};
</script>
