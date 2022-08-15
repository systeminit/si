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
          <SiDropdownItem class="text-sm" :checked="true"> </SiDropdownItem>
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
import { onMounted, onUpdated, provide, ref } from "vue";
import { DotsVerticalIcon } from "@heroicons/vue/outline";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";

const props = withDefaults(
  defineProps<{
    selectedIndex?: number;
    onChange?: (_index: number) => void;
    tabListClasses?: string;
    startMargin?: number;
    afterMargin?: number;
  }>(),
  {
    selectedIndex: undefined,
    onChange: undefined,
    tabListClasses:
      "h-11 flex shrink-0 w-full bg-white dark:bg-neutral-800 sticky top-0 z-5 overflow-hidden",
    startMargin: 0,
    afterMargin: 0,
  },
);

const tabList = ref();
const endSpace = ref();
const dropDown = ref();
const updateDropDown = () => {
  const tabListEl = tabList.value.$el;
  const endSpaceEl = endSpace.value;
  const dropDownEl = dropDown.value.$el;

  console.log(
    tabList.value?.$el.scrollWidth + " / " + tabList.value?.$el.clientWidth,
  );
  console.log(dropDownEl);

  if (tabListEl !== undefined) {
    endSpaceEl.classList = "";
    dropDownEl.classList = "";
    if (tabListEl.scrollWidth > tabListEl.clientWidth) {
      console.log("OVERFLOW!");
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
      dropDownEl.classList.remove("hidden");
    } else {
      console.log("NO OVERFLOW!");
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

provide("afterMargin", props.afterMargin);
</script>
