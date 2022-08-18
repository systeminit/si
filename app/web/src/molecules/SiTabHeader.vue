<template>
  <!-- selectedToFront ? (selected ? 'order-1' : 'order-2') : '' -->
  <Tab
    v-slot="{ selected }"
    class="focus:outline-none whitespace-nowrap"
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
        :class="
          classesF + ' ' + (selected ? selectedClassesF : defaultClassesF)
        "
      >
        <slot />
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
import { inject } from "vue";

const props = defineProps<{
  classes?: string;
  defaultClasses?: string;
  selectedClasses?: string;
}>();

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

const afterMargin = inject("afterMargin", 0);
const selectedToFront = inject("selectedTabToFront", false);
const classesF = inject("tabClasses", props.classes);
const defaultClassesF = inject("defaultTabClasses", props.defaultClasses);
const selectedClassesF = inject("selectedTabClasses", props.selectedClasses);
</script>
