<template>
  <TabGroup :selected-index="selectedIndex" @change="props.onChange">
    <slot />
    <div class="w-full h-11 relative">
      <TabList ref="tabList" :class="tabListClasses">
        <div
          v-if="startMargin > 0"
          class="border-b border-neutral-300 dark:border-neutral-600"
          :class="'w-' + startMargin"
        ></div>
        <slot name="tabs" />
        <div ref="endSpace"></div>
      </TabList>
      <SiBarButton
        ref="dropDown"
        :padding-x="2"
        :hover-effect="false"
        dropdown-classes="-right-0 z-100"
        ><DotsVerticalIcon class="w-6" />
        <template #dropdownContent>
          <SiDropdownItem class="text-sm" :checked="true">TEST</SiDropdownItem>
        </template>
      </SiBarButton>
    </div>
    <TabPanels as="template">
      <slot name="panels" />
    </TabPanels>
  </TabGroup>
</template>

<script setup lang="ts">
import { TabGroup, TabPanels, TabList } from "@headlessui/vue";
import {
  onBeforeUnmount,
  onMounted,
  onUpdated,
  provide,
  ref,
  useSlots,
} from "vue";
import { DotsVerticalIcon } from "@heroicons/vue/outline";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import _ from "lodash";

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
  }>(),
  {
    selectedIndex: undefined,
    onChange: undefined,
    tabListClasses:
      "h-11 flex shrink-0 w-full bg-white dark:bg-neutral-800 sticky top-0 z-5 overflow-hidden",
    tabClasses:
      "border-x border-t border-x-neutral-300 border-t-neutral-300 dark:border-x-neutral-600 dark:border-t-neutral-600 h-11 px-2 text-sm inline-flex items-center rounded-t",
    defaultTabClasses:
      "text-gray-400 border-b border-neutral-300 dark:border-neutral-600 font-medium",
    selectedTabClasses:
      "border-b-white dark:border-b-neutral-800 border-b text-action-700 dark:text-action-300 font-bold",
    startMargin: 0,
    afterMargin: 0,
  },
);

const slots = useSlots();

const tabList = ref();
const endSpace = ref();
const dropDown = ref();
const updateDropDown = () => {
  const tabListEl = tabList.value?.$el;
  const dropDownEl = dropDown.value?.$el;
  const endSpaceEl = endSpace.value;

  // console.log(
  //   tabList.value?.$el.scrollWidth + " / " + tabList.value?.$el.clientWidth,
  // );
  // console.log(dropDownEl);

  if (tabListEl !== undefined) {
    endSpaceEl.classList = "";
    dropDownEl.classList = "";
    if (tabListEl.scrollWidth > tabListEl.clientWidth) {
      //console.log("OVERFLOW!");
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
      //console.log("NO OVERFLOW!");
      endSpaceEl.classList.add("grow");
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
</script>
