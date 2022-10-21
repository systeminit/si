<template>
  <div
    class="si-tab-group absolute w-full h-full flex flex-col overflow-hidden"
  >
    <TabGroup :selected-index="selectedIndex" @change="props.onChange">
      <slot />
      <div
        :class="
          clsx(
            'si-tab-group__header',
            'w-full h-11 relative flex-shrink-0',
            topMargin > 0 ? `mt-${topMargin}` : '',
          )
        "
      >
        <TabList
          ref="tabList"
          :class="clsx('si-tab-group__tabs', tabListClasses)"
        >
          <div
            v-if="noStartMargin === false"
            :class="selectedTabToFront ? ' order-first' : ''"
            class="w-2 border-b border-neutral-300 dark:border-neutral-600"
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
          dropdown-classes="right-0 overflow-x-hidden max-w-xs text-left text-ellipsis max-h-96 overflow-y-auto"
          dropdown-item-classes="text-left text-ellipsis"
        >
          <Icon name="dots-vertical" />
          <template #dropdownContent>
            <slot name="dropdownitems" />
          </template>
        </SiBarButton>
      </div>
      <TabPanels class="si-tab-group__body flex-grow overflow-auto relative">
        <slot name="panels" />
      </TabPanels>
    </TabGroup>
  </div>
</template>

<script lang="ts" setup>
import { TabGroup, TabList, TabPanels } from "@headlessui/vue";
import { onBeforeUnmount, onMounted, onUpdated, provide, ref } from "vue";
import _ from "lodash";
import clsx from "clsx";
import SiBarButton from "@/molecules/SiBarButton.vue";
import Icon from "@/ui-lib/Icon.vue";

const props = withDefaults(
  defineProps<{
    selectedIndex?: number;
    onChange?: (_index: number) => void;
    tabListClasses?: string;
    tabClasses?: string;
    defaultTabClasses?: string;
    selectedTabClasses?: string;
    noStartMargin?: boolean;
    noAfterMargin?: boolean;
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
      "border-x border-t border-x-neutral-300 border-t-neutral-300 dark:border-x-neutral-600 dark:border-t-neutral-600 h-11 px-2 text-sm inline-flex items-center rounded-t group-hover:border-shade-100 dark:group-hover:border-shade-0",
    defaultTabClasses:
      "text-neutral-400 border-b border-neutral-300 dark:border-neutral-600 font-medium",
    selectedTabClasses:
      "border-b-white dark:border-b-neutral-800 border-b text-action-700 dark:text-action-300 font-bold",
    topMargin: 2,
    noStartMargin: false,
    noAfterMargin: false,
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

provide("noAfterMargin", props.noAfterMargin);
provide("tabClasses", props.tabClasses);
provide("defaultTabClasses", props.defaultTabClasses);
provide("selectedTabClasses", props.selectedTabClasses);
provide("selectedTabToFront", props.selectedTabToFront);
provide("tabWidthMaximum", props.tabWidthMaximum);
</script>
