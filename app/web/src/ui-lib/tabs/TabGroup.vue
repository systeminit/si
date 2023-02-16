<template>
  <div class="absolute inset-0 flex flex-col">
    <!-- TabGroupItems go in this slot but are not rendered here. -->
    <div class="hidden"><slot /></div>

    <!-- special slot for when no tabs exist - mostly useful for dynamic tab situations -->
    <slot v-if="isNoTabs" name="noTabs">No tabs.</slot>

    <template v-else>
      <!-- This div holds the actual tabs -->
      <div
        ref="tabContainerRef"
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
        <template v-for="tab in orderedTabs" :key="tab.props.slug">
          <a
            :ref="
              (el) => {
                tabRefs[tab.props.slug] = el as HTMLElement;
              }
            "
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
            class="border-b border-neutral-300 dark:border-neutral-600 w-2xs shrink-0"
          />
        </template>
        <div
          class="flex-grow border-b border-neutral-300 dark:border-neutral-600 order-last"
        ></div>

        <div
          v-if="showOverflowDropdown"
          :class="
            clsx(
              'border border-neutral-300 dark:border-neutral-600 h-full px-xs items-center flex absolute right-0 top-0 z-10 cursor-pointer',
              overflowMenuRef?.isOpen
                ? 'bg-neutral-200 dark:bg-black'
                : 'bg-white dark:bg-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-900', // TODO(Wendy) - add this mouseover effect to all tabs
            )
          "
          @click="overflowMenuRef?.open"
        >
          <Icon name="dots-vertical" />
        </div>
        <DropdownMenu ref="overflowMenuRef" force-align-right>
          <DropdownMenuItem
            v-for="tab in orderedTabs"
            :key="tab.props.slug"
            @select="selectTab(tab.props.slug)"
          >
            {{ tab.props.label }}
          </DropdownMenuItem>
        </DropdownMenu>
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
  onUpdated,
  onBeforeUnmount,
} from "vue";
import Icon from "../icons/Icon.vue";
import DropdownMenu from "../menus/DropdownMenu.vue";
import DropdownMenuItem from "../menus/DropdownMenuItem.vue";
import { themeClasses } from "../theme_tools";
import { TabGroupItemDefinition } from "./TabGroupItem.vue";

const showOverflowDropdown = ref(false);
const overflowMenuRef = ref();
const tabContainerRef = ref();
const tabRefs = ref({} as Record<string, HTMLElement | null>);

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
const orderedTabSlugs = ref<string[]>([]);
const orderedTabs = computed(() =>
  _.map(orderedTabSlugs.value, (slug) => tabs[slug]),
);
const selectedTabSlug = ref<string>();

function registerTab(slug: string, component: TabGroupItemDefinition) {
  tabs[slug] = component;
  orderedTabSlugs.value = [...orderedTabSlugs.value, slug];
  // refreshSortedTabSlugs();
  refreshSettingsFromTabs();
}
function unregisterTab(slug: string) {
  delete tabs[slug];
  orderedTabSlugs.value = _.without(orderedTabSlugs.value, slug);
  // refreshSortedTabSlugs();
  refreshSettingsFromTabs();
  autoSelectTab();
}

function refreshSettingsFromTabs() {
  // currently there are no settings here - any child settings to set on the parent would go here
}

function selectTab(slug?: string) {
  if (selectedTabSlug.value === slug) return;

  // select the tab
  if (slug && tabs[slug]) selectedTabSlug.value = slug;
  else selectedTabSlug.value = undefined;
  emit("update:selectedTab", selectedTabSlug.value);

  // adjust the tab position if it is offscreen
  if (selectedTabSlug.value) {
    const tabEl = tabRefs.value[selectedTabSlug.value];
    if (tabEl) {
      const tabElRect = tabEl.getBoundingClientRect();
      const tabContainerRect = tabContainerRef.value.getBoundingClientRect();
      if (tabElRect.right > tabContainerRect.right) {
        orderedTabSlugs.value = _.orderBy(orderedTabSlugs.value, (slug) =>
          slug === selectedTabSlug.value ? 0 : 1,
        );
      }
    }
  }
}

function autoSelectTab() {
  if (isNoTabs.value) {
    // can't select anything if there are no tabs
    selectTab();
    return;
  } else if (selectedTabSlug.value && tabs[selectedTabSlug.value]) {
    // currently selected tab is all good
    return;
  } else if (props.startSelectedTabSlug && tabs[props.startSelectedTabSlug]) {
    // select the starting tab if it exists
    // TODO: probably only want to do this in some cases (like initial load)
    selectTab(props.startSelectedTabSlug);
  } else {
    selectTab(_.keys(tabs)[0]);
  }
}

onMounted(autoSelectTab);

function fixOverflowDropdown() {
  const tabListEl = tabContainerRef.value;
  if (!tabListEl) return;
  showOverflowDropdown.value = tabListEl.scrollWidth > tabListEl.clientWidth;
}
onMounted(fixOverflowDropdown);
onUpdated(fixOverflowDropdown);
const debounceForResize = _.debounce(fixOverflowDropdown, 50);
const resizeObserver = new ResizeObserver(debounceForResize);
watch(tabContainerRef, () => {
  if (tabContainerRef.value) {
    resizeObserver.observe(tabContainerRef.value);
  } else {
    resizeObserver.unobserve(tabContainerRef.value);
  }
});
onBeforeUnmount(() => {
  if (tabContainerRef.value) {
    resizeObserver.unobserve(tabContainerRef.value);
  }
});

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
