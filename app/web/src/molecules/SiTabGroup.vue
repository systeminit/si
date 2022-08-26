<template>
  <TabGroup :selected-index="selectedIndex" @change="props.onChange">
    <slot />
    <div
      :class="topMargin > 0 ? `mt-${topMargin}` : ''"
      class="w-full h-11 relative"
    >
      <TabList ref="tabList" :class="tabListClasses">
        <div
          v-if="startMargin > 0"
          :class="
            'w-' + startMargin + (selectedTabToFront ? ' order-first' : '')
          "
          class="border-b border-neutral-300 dark:border-neutral-600"
        ></div>
        <slot name="tabs" />
        <div ref="endSpace"></div>
      </TabList>
      <SiBarButton
        ref="dropDown"
        :dropdown-item-show-suffix="false"
        :hover-effect="false"
        :navbar="false"
        :padding-x="2"
        dropdown-classes="right-0 overflow-hidden max-w-xs text-left text-ellipsis"
        dropdown-item-classes="text-left text-ellipsis"
      >
        <DotsVerticalIcon class="w-6" />
        <template #dropdownContent>
          <slot name="dropdownitems" />
        </template>
      </SiBarButton>
    </div>
    <TabPanels as="template">
      <slot name="panels" />
    </TabPanels>
  </TabGroup>
</template>

<script lang="ts" setup>
import { TabGroup, TabList, TabPanels } from "@headlessui/vue";
import { onBeforeUnmount, onMounted, onUpdated, provide, ref } from "vue";
import { DotsVerticalIcon } from "@heroicons/vue/outline";
import _ from "lodash";
import SiBarButton from "@/molecules/SiBarButton.vue";

const props = withDefaults(
  defineProps<{
    selectedIndex?: number;
    onChange?: (_index: number) => void;
    tabListClasses?: string;
    tabClasses?: string;
    defaultTabClasses?: string;
    selectedTabClasses?: string;
    startMargin?: number;
    afterMargin?: number;
    topMargin?: number;
    selectedTabToFront?: boolean;
    tabWidthMaximum?: number;
  }>(),
  {
    selectedIndex: undefined,
    onChange: undefined,
    tabListClasses:
      "h-11 flex shrink-0 w-full bg-white dark:bg-neutral-800 sticky top-0 z-5 overflow-hidden",
    tabClasses:
      "border-x border-t border-x-neutral-300 border-t-neutral-300 dark:border-x-neutral-600 dark:border-t-neutral-600 h-11 px-2 text-sm inline-flex items-center rounded-t",
    defaultTabClasses:
      "text-neutral-400 border-b border-neutral-300 dark:border-neutral-600 font-medium",
    selectedTabClasses:
      "border-b-white dark:border-b-neutral-800 border-b text-action-700 dark:text-action-300 font-bold",
    topMargin: 2,
    startMargin: 4,
    afterMargin: 2,
    selectedTabToFront: false,
    tabWidthMaximum: 0,
  },
);

const tabList = ref();
const endSpace = ref();
const dropDown = ref();
const updateDropDown = () => {
  const tabListEl = tabList.value?.$el;
  const dropDownEl = dropDown.value?.$el;
  const endSpaceEl = endSpace.value;

  if (tabListEl !== undefined) {
    endSpaceEl.classList = "";
    dropDownEl.classList = "";
    if (tabListEl.scrollWidth > tabListEl.clientWidth) {
      endSpaceEl.classList.add("hidden");
      dropDownEl.classList.add(
        "border",
        "border-neutral-300",
        "dark:border-neutral-600",
        "w-11",
        "h-11",
        "absolute",
        "right-0",
        "top-0",
        "z-100",
        "bg-white",
        "dark:bg-neutral-800",
        "text-center",
      );
    } else {
      endSpaceEl.classList.add("grow");
      if (props.selectedTabToFront) {
        endSpaceEl.classList.add("order-last");
      }
      dropDownEl.classList.add("hidden");
    }
    endSpaceEl.classList.add(
      "border-b",
      "border-neutral-300",
      "dark:border-neutral-600",
    );
  }
};

onMounted(updateDropDown);
onUpdated(updateDropDown);

const debounceForResize = _.debounce(updateDropDown, 50);
const resizeObserver = new ResizeObserver(debounceForResize);

onMounted(() => {
  resizeObserver.observe(tabList.value?.$el);
});

onBeforeUnmount(() => {
  resizeObserver.unobserve(tabList.value?.$el);
});

provide("afterMargin", props.afterMargin);
provide("tabClasses", props.tabClasses);
provide("defaultTabClasses", props.defaultTabClasses);
provide("selectedTabClasses", props.selectedTabClasses);
provide("selectedTabToFront", props.selectedTabToFront);
provide("tabWidthMaximum", props.tabWidthMaximum);
</script>
