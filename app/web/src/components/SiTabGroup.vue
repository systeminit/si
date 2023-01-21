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
          ref="tabListRef"
          :class="clsx('si-tab-group__tabs', tabListClasses)"
        >
          <div
            v-if="noStartMargin === false"
            :class="selectedTabToFront ? ' order-first' : ''"
            class="w-2 border-b border-neutral-300 dark:border-neutral-600"
          ></div>
          <slot name="tabs" />
          <div
            class="flex-grow border-b border-neutral-300 dark:border-neutral-600 order-last"
          ></div>
        </TabList>

        <div
          v-if="showOverflowDropdown"
          :class="
            clsx(
              'border border-neutral-300 dark:border-neutral-600 h-full px-xs items-center flex absolute right-0 top-0 z-10',
              overflowMenuRef?.isOpen
                ? 'bg-neutral-200 dark:bg-black'
                : 'bg-white dark:bg-neutral-800',
            )
          "
          @click="overflowMenuRef?.open"
        >
          <Icon name="dots-vertical" />
        </div>

        <DropdownMenu ref="overflowMenuRef">
          <slot name="dropdownContent"></slot>
        </DropdownMenu>
      </div>
      <TabPanels class="si-tab-group__body flex-grow overflow-auto relative">
        <slot name="panels" />
      </TabPanels>
    </TabGroup>
  </div>
</template>

<script lang="ts" setup>
import { TabGroup, TabList, TabPanels } from "@headlessui/vue";
import {
  onBeforeUnmount,
  onMounted,
  onUpdated,
  provide,
  ref,
  useSlots,
} from "vue";
import _ from "lodash";
import clsx from "clsx";
import Icon from "@/ui-lib/icons/Icon.vue";
import DropdownMenu from "@/ui-lib/menus/DropdownMenu.vue";

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

const overflowMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const showOverflowDropdown = ref(false);

const slots = useSlots();

const tabListRef = ref();
function fixOverflowDropdown() {
  if (!slots.dropdownContent) return;
  const tabListEl = tabListRef.value?.$el;
  if (!tabListEl) return;

  showOverflowDropdown.value = tabListEl.scrollWidth > tabListEl.clientWidth;
}

onMounted(fixOverflowDropdown);
onUpdated(fixOverflowDropdown);

const debounceForResize = _.debounce(fixOverflowDropdown, 50);
const resizeObserver = new ResizeObserver(debounceForResize);

onMounted(() => {
  resizeObserver.observe(tabListRef.value?.$el);
});

onBeforeUnmount(() => {
  resizeObserver.unobserve(tabListRef.value?.$el);
});

provide("noAfterMargin", props.noAfterMargin);
provide("tabClasses", props.tabClasses);
provide("defaultTabClasses", props.defaultTabClasses);
provide("selectedTabClasses", props.selectedTabClasses);
provide("selectedTabToFront", props.selectedTabToFront);
provide("tabWidthMaximum", props.tabWidthMaximum);
</script>
