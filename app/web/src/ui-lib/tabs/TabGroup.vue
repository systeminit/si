<template>
  <div ref="internalRef">
    <!-- TabGroupItems go in this slot but are not rendered here. -->
    <div class="hidden"><slot /></div>

    <slot v-if="isNoTabs" name="noTabs">No tabs.</slot>
    <div v-else>
      <div
        :class="
          clsx(
            'w-full h-11 relative flex-shrink-0 flex flex-row',
            topMargin > 0 ? `mt-${topMargin}` : '',
            'h-11 flex shrink-0 w-full bg-white dark:bg-neutral-800 sticky top-0 z-5 overflow-hidden',
          )
        "
      >
        <div
          v-if="noStartMargin === false"
          class="w-2 border-b border-neutral-300 dark:border-neutral-600"
        ></div>
        <template v-for="tab in tabs" :key="tab.props.slug">
          <a
            href="#"
            :class="
              clsx(
                'focus:outline-none whitespace-nowrap',
                'text-neutral-400 border-b border-neutral-300 dark:border-neutral-600 border-x border-t border-x-neutral-300 border-t-neutral-300 dark:border-x-neutral-600 dark:border-t-neutral-600',
                'h-11 px-2 text-sm inline-flex items-center rounded-t group-hover:border-shade-100 dark:group-hover:border-shade-0',
                tab.props.slug === selectedTabSlug
                  ? 'border-b-white dark:border-b-neutral-800 border-b text-action-700 dark:text-action-300 font-bold'
                  : themeClasses(
                      'hover:text-neutral-400 font-medium',
                      'hover:text-neutral-300 font-medium',
                    ),
              )
            "
            @click.prevent="selectTab(tab.props.slug)"
          >
            <template v-if="tab.slots.label">
              <component :is="tab.slots.label" />
            </template>
            <template v-else>{{ tab.props.label }}</template>
          </a>
          <div
            v-if="noAfterMargin === false"
            class="border-b border-neutral-300 dark:border-neutral-600 w-2"
          />
        </template>
        <div
          class="flex-grow border-b border-neutral-300 dark:border-neutral-600 order-last"
        ></div>
      </div>

      <div v-if="selectedTabSlug && tabs[selectedTabSlug]">
        <component :is="tabs[selectedTabSlug].slots.default" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
type TabGroupContext = {
  selectedTabSlug: Ref<string | undefined>;
  registerTab(id: string, component: TabGroupItemDefinition): void;
  unregisterTab(id: string): void;
  selectTab(id?: string): void;
};

export const TabGroupContextInjectionKey: InjectionKey<TabGroupContext> =
  Symbol("TabGroupContext");

export function useTabGroupContext() {
  const ctx = inject(TabGroupContextInjectionKey, null);
  if (!ctx)
    throw new Error("<TabGroupItem> should only be used within a <TabGroup>");
  return ctx;
}
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import clsx from "clsx";
import _ from "lodash";
import {
  ref,
  Ref,
  InjectionKey,
  inject,
  reactive,
  computed,
  provide,
  onMounted,
} from "vue";
import { themeClasses } from "../theme_tools";
import { TabGroupItemDefinition } from "./TabGroupItem.vue";

const internalRef = ref();

const props = defineProps({
  startSelectedTabSlug: { type: String },
  topMargin: { type: Number, default: 2 },
  noStartMargin: { type: Boolean, default: false },
  noAfterMargin: { type: Boolean, default: false },
});

const isNoTabs = computed(() => !_.keys(tabs).length);

const tabs = reactive({} as Record<string, TabGroupItemDefinition>);
// const sortedTabSlugs = ref<string[]>([]);
const selectedTabSlug = ref<string>();

function registerTab(slug: string, component: TabGroupItemDefinition) {
  tabs[slug] = component;
  // refreshSortedTabSlugs();
  refreshSettingsFromTabs();
}
function unregisterTab(slug: string) {
  delete tabs[slug];
  // refreshSortedTabSlugs();
  refreshSettingsFromTabs();
}

function refreshSettingsFromTabs() {
  // currently there are no settings here - any child settings to set on the parent would go here
}

function selectTab(slug?: string) {
  if (slug && tabs[slug]) selectedTabSlug.value = slug;
  else selectedTabSlug.value = undefined;
}

function autoSelectTab() {
  if (isNoTabs.value) {
    // can't select anything if there are no tabs
    selectedTabSlug.value = undefined;
    return;
  } else if (selectedTabSlug.value && tabs[selectedTabSlug.value]) {
    // currently selected tab is all good
    return;
  } else if (props.startSelectedTabSlug && tabs[props.startSelectedTabSlug]) {
    // select the starting tab if it exists
    selectedTabSlug.value = props.startSelectedTabSlug;
  } else {
    // TODO(Wendy) - more ordering logic for which tab to select
    selectedTabSlug.value = _.keys(tabs)[0];
  }
}

onMounted(autoSelectTab);

// Externally exposed info /////////////////////////////////////////////////////////////////////////////////////////

// this object gets provided to the child DropDownMenuItems
const context = {
  selectedTabSlug,
  registerTab,
  unregisterTab,
  selectTab,
};
provide(TabGroupContextInjectionKey, context);
</script>
