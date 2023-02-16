<template>
  <div class="absolute inset-0 flex flex-col">
    <!-- TabGroupItems go in this slot but are not rendered here. -->
    <div class="hidden"><slot /></div>

    <!-- special slot for when no tabs exist - mostly useful for dynamic tab situations -->
    <slot v-if="isNoTabs" name="noTabs">No tabs.</slot>

    <template v-else>
      <!-- This div holds the actual tabs -->
      <div
        :class="
          clsx(
            'w-full h-11 relative flex flex-row shrink-0 bg-white dark:bg-neutral-800 overflow-hidden mt-2',
          )
        "
      >
        <div
          v-if="firstTabMarginLeft && firstTabMarginLeft !== 'none'"
          :class="
            clsx(
              {
                '2xs': 'w-2xs',
                xs: 'w-xs',
                sm: 'w-sm',
                md: 'w-md',
              }[firstTabMarginLeft],
              'border-b border-neutral-300 dark:border-neutral-600',
            )
          "
        />
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
            <button
              v-if="closeable"
              class="inline-block rounded-full text-neutral-400 ml-1"
              :class="
                clsx(
                  themeClasses(
                    'hover:text-white hover:bg-neutral-400',
                    'hover:text-neutral-800 hover:bg-neutral-400',
                  ),
                )
              "
              @click.stop="emit('closeTab', tab.props.slug)"
            >
              <Icon name="x" size="xs" />
            </button>
          </a>
          <div
            class="border-b border-neutral-300 dark:border-neutral-600 w-2xs"
          />
        </template>
        <div
          class="flex-grow border-b border-neutral-300 dark:border-neutral-600 order-last"
        ></div>
      </div>

      <!-- Here we actually render the tab content of the current tab -->
      <template v-if="selectedTabSlug && tabs[selectedTabSlug]">
        <!-- extra slots to make it easy to have non-scrolling content above/below scrolling area -->
        <div v-if="tabs[selectedTabSlug].slots.stickyTop">
          <component :is="tabs[selectedTabSlug].slots.stickyTop" />
        </div>
        <div class="overflow-auto flex-grow">
          <component :is="tabs[selectedTabSlug].slots.default" />
        </div>
        <div v-if="tabs[selectedTabSlug].slots.stickyBottom">
          <component :is="tabs[selectedTabSlug].slots.stickyBottom" />
        </div>
      </template>
    </template>
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
  PropType,
  watch,
} from "vue";
import Icon from "../icons/Icon.vue";
import { themeClasses } from "../theme_tools";
import { TabGroupItemDefinition } from "./TabGroupItem.vue";

const props = defineProps({
  startSelectedTabSlug: { type: String },
  closeable: { type: Boolean, default: false },
  firstTabMarginLeft: {
    type: String as PropType<"none" | "2xs" | "xs" | "sm" | "md">,
    default: "xs",
  },
});

const emit = defineEmits<{
  (e: "closeTab", slug: string): void;
  (e: "update:selectedTab", slug: string | undefined): void;
}>();

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
  autoSelectTab();
}

function refreshSettingsFromTabs() {
  // currently there are no settings here - any child settings to set on the parent would go here
}

function selectTab(slug?: string) {
  if (selectedTabSlug.value === slug) return;
  if (slug && tabs[slug]) selectedTabSlug.value = slug;
  else selectedTabSlug.value = undefined;
}

watch(selectedTabSlug, () => {
  emit("update:selectedTab", selectedTabSlug.value);
});

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
    // TODO: probably only want to do this in some cases (like initial load)
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

defineExpose({ selectTab });
</script>
