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
          ? selected && $el && $el.nextElementSibling
            ? moveTabToFrontIfOverflowing($el.nextElementSibling)
            : 'order-2'
          : ''
      "
    >
      <span
        class="w-full flex items-center"
        :class="
          clsx([
            classesF,
            selected ? selectedClassesF : defaultClassesF,
            selected
              ? ''
              : themeClasses(
                  'hover:text-neutral-400',
                  'hover:text-neutral-300',
                ),
          ])
        "
      >
        <span class="overflow-hidden text-ellipsis">
          <slot />
        </span>
        <span class="flex">
          <slot name="icon" />
        </span>
      </span>
    </button>
  </Tab>
  <div
    v-if="noAfterMargin === false"
    class="border-b border-neutral-300 dark:border-neutral-600 w-2"
    :class="selectedToFront ? 'order-2' : ''"
  ></div>
</template>

<script setup lang="ts">
import { Tab } from "@headlessui/vue";
import { computed, inject } from "vue";
import clsx from "clsx";
import { themeClasses } from "@/ui-lib/theme_tools";

const props = defineProps<{
  classes?: string;
  defaultClasses?: string;
  selectedClasses?: string;
}>();

const noAfterMargin = inject("noAfterMargin", false);
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
