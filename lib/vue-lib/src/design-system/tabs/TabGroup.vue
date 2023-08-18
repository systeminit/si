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
                      'hover:text-neutral-400 font-medium hover:bg-neutral-100',
                      'hover:text-neutral-300 font-medium hover:bg-neutral-900',
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
              v-if="closeable && !tab.props.uncloseable"
              class="inline-block rounded-full text-neutral-400 ml-1"
              :class="
                clsx(
                  themeClasses(
                    'hover:text-white hover:bg-neutral-400',
                    'hover:text-neutral-800 hover:bg-neutral-400',
                  ),
                )
              "
              @click.prevent.stop="emit('closeTab', tab.props.slug)"
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
          ref="overflowDropdownButtonRef"
          :class="
            clsx(
              'border border-neutral-300 dark:border-neutral-600 h-full px-xs items-center flex absolute right-0 top-0 z-10 cursor-pointer',
              'bg-white dark:bg-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-900',
            )
          "
          @click="overflowMenuRef?.open"
        >
          <Icon name="dots-vertical" />
        </div>
        <DropdownMenu ref="overflowMenuRef" forceAlignRight>
          <DropdownMenuItem
            v-for="tab in orderedTabs"
            :key="tab.props.slug"
            @select="selectTab(tab.props.slug)"
          >
            <!-- TODO: we may need another slot for rendering custom labels in the overflow menu -->
            <component :is="tab.slots.label" v-if="tab.slots.label" />
            <span v-else>{{ tab.props.label }}</span>
          </DropdownMenuItem>
        </DropdownMenu>
      </div>

      <!-- the tabgroup item uses a teleport to render its default slot content here if active -->
      <TeleportTarget
        :id="teleportId"
        class="overflow-auto flex-grow relative"
      />
    </template>
  </div>
</template>

<script lang="ts">
type TabGroupContext = {
  selectedTabSlug: Ref<string | undefined>;
  registerTab(id: string, component: TabGroupItemDefinition): void;
  unregisterTab(id: string): void;
  selectTab(id?: string): void;
  tabExists(id?: string): boolean;
  teleportId: string;
};

export const TabGroupContextInjectionKey: InjectionKey<TabGroupContext> =
  Symbol("TabGroupContext");

export function useTabGroupContext() {
  const ctx = inject(TabGroupContextInjectionKey, null);
  if (!ctx)
    throw new Error("<TabGroupItem> should only be used within a <TabGroup>");
  return ctx;
}

let tabGroupCounter = 1;
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import clsx from "clsx";
import * as _ from "lodash-es";
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
  nextTick,
} from "vue";
import posthog from "posthog-js";
import { TeleportTarget } from "vue-safe-teleport";
import { Icon, DropdownMenu, DropdownMenuItem } from "..";
import { themeClasses } from "../utils/theme_tools";
import { TabGroupItemDefinition } from "./TabGroupItem.vue";

const unmounting = ref(false);
const showOverflowDropdown = ref(false);
const overflowMenuRef = ref();
const overflowDropdownButtonRef = ref();
const tabContainerRef = ref();
const tabRefs = ref({} as Record<string, HTMLElement | null>);

const props = defineProps({
  startSelectedTabSlug: { type: String },
  rememberSelectedTabKey: { type: String },
  closeable: { type: Boolean, default: false },
  firstTabMarginLeft: {
    type: String as PropType<"none" | "2xs" | "xs" | "sm" | "md">,
    default: "xs",
  },
  trackingSlug: String,
});

const emit = defineEmits<{
  (e: "closeTab", slug: string): void;
  (e: "update:selectedTab", slug: string | undefined): void;
}>();

const teleportId = `tabs-portal-${tabGroupCounter++}`;

const isNoTabs = computed(() => !_.keys(tabs).length);

const tabs = reactive({} as Record<string, TabGroupItemDefinition>);
const orderedTabSlugs = ref<string[]>([]);
const orderedTabs = computed(
  () =>
    _.map(orderedTabSlugs.value, (slug) => tabs[slug]).filter(
      (tab) => !!tab,
    ) as TabGroupItemDefinition[],
);
const selectedTabSlug = ref<string>();

function registerTab(slug: string, component: TabGroupItemDefinition) {
  tabs[slug] = component;
  orderedTabSlugs.value = [...orderedTabSlugs.value, slug];
  // refreshSortedTabSlugs();
  refreshSettingsFromTabs();
  // if this is the first tab we are registering, we'll autoselect on the next tick
  if (_.keys(tabs).length === 1) {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    nextTick(() => {
      autoSelectTab(true);
    });
  }

  if (pendingTabSlug.value && pendingTabSlug.value === slug) {
    selectTab(slug);
  }
}
function unregisterTab(slug: string) {
  if (unmounting.value) return;
  orderedTabSlugs.value = _.without(orderedTabSlugs.value, slug);
  delete tabs[slug];
  // refreshSortedTabSlugs();
  refreshSettingsFromTabs();
  if (isNoTabs.value) {
    emit("update:selectedTab", undefined);
  } else {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    nextTick(() => {
      autoSelectTab();
    });
  }
}

function refreshSettingsFromTabs() {
  // currently there are no settings here - any child settings to set on the parent would go here
}

function tabExists(slug?: string) {
  return !!(slug && tabs[slug]);
}

const pendingTabSlug = ref<string | undefined>();
const lastSelectedTabIndex = ref(0);
function selectTab(slug?: string) {
  if (unmounting.value) return;
  if (selectedTabSlug.value === slug) return;

  // if selecting no tab, autoselect
  if (!slug) {
    autoSelectTab();
    return;
  }

  // select the tab
  if (slug && tabs[slug]) {
    selectedTabSlug.value = slug;
    pendingTabSlug.value = undefined;
  } else {
    // If the tab is not yet present, we mark this as the pending tab slug. When
    // registerTab is called with a matching slug, that tab will be selected.
    // Any other tab selection clears the pending tab slug
    selectedTabSlug.value = undefined;
    pendingTabSlug.value = slug;
  }

  lastSelectedTabIndex.value = _.indexOf(
    orderedTabSlugs.value,
    selectedTabSlug.value,
  );

  if (props.trackingSlug) {
    posthog.capture("wa-tab_selected", {
      groupSlug: props.trackingSlug,
      tabSlug: selectedTabSlug.value,
    });
  }

  if (selectedTabSlug.value) {
    if (rememberLastTabStorageKey.value) {
      window.localStorage.setItem(
        rememberLastTabStorageKey.value,
        selectedTabSlug.value,
      );
    }
    // adjust the tab position if it is offscreen
    const tabEl = tabRefs.value[selectedTabSlug.value];
    if (tabEl) {
      const tabElRect = tabEl.getBoundingClientRect();
      const tabContainerRect = tabContainerRef.value.getBoundingClientRect();
      // Need to account for the overflow dropdown button!
      const overflowButtonWidth = overflowDropdownButtonRef.value
        ? overflowDropdownButtonRef.value.getBoundingClientRect().width
        : 0;
      const limit = tabContainerRect.right - overflowButtonWidth;
      if (tabElRect.right > limit) {
        orderedTabSlugs.value = _.orderBy(orderedTabSlugs.value, (slug) =>
          slug === selectedTabSlug.value ? 0 : 1,
        );
      }
    }
  }

  // emit new selected tab to parent in case it needs it, for example to sync the URL
  emit("update:selectedTab", selectedTabSlug.value);
}

const rememberLastTabStorageKey = computed(() => {
  if (props.rememberSelectedTabKey) {
    return `tab_group_${props.rememberSelectedTabKey}`;
  } else {
    return false;
  }
});

function autoSelectTab(isInitialSelection = false) {
  pendingTabSlug.value = undefined;
  if (isNoTabs.value) {
    // can't select anything if there are no tabs
    // selectTab();
    return;
  }
  if (selectedTabSlug.value && tabs[selectedTabSlug.value]) {
    // currently selected tab is all good
    return;
  } else if (
    isInitialSelection &&
    props.startSelectedTabSlug &&
    tabs[props.startSelectedTabSlug]
  ) {
    // select the starting tab if it exists
    // TODO: probably only want to do this in some cases (like initial load)
    selectTab(props.startSelectedTabSlug);
    return;
  } else if (isInitialSelection && rememberLastTabStorageKey.value) {
    const slug = window.localStorage.getItem(rememberLastTabStorageKey.value);
    if (slug && tabs[slug]) {
      selectTab(slug);
      return;
    }
  }
  // fallback to just autoselecting the tab next the last one selected
  let newIndex = (lastSelectedTabIndex.value || 0) - 1;
  if (newIndex < 0) newIndex = 0;
  selectTab(orderedTabSlugs.value[newIndex]);
}

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
    resizeObserver.disconnect();
  }
});
onBeforeUnmount(() => {
  unmounting.value = true;
  resizeObserver.disconnect();
});

// These style classes have to be handled here to avoid a rerendering bug with Codemirror
watch(
  () => overflowMenuRef.value?.isOpen,
  () => {
    if (!overflowMenuRef.value || !overflowDropdownButtonRef.value) return;

    const classes =
      "bg-white dark:bg-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-900".split(
        " ",
      );
    if (overflowMenuRef.value.isOpen) {
      overflowDropdownButtonRef.value.classList.add(
        "bg-neutral-200",
        "dark:bg-black",
      );
      overflowDropdownButtonRef.value.classList.remove(...classes);
    } else {
      overflowDropdownButtonRef.value.classList.add(...classes);
      overflowDropdownButtonRef.value.classList.remove(
        "bg-neutral-200",
        "dark:bg-black",
      );
    }
  },
);

// Externally exposed info /////////////////////////////////////////////////////////////////////////////////////////

// this object gets provided to the child DropDownMenuItems
const context = {
  selectedTabSlug,
  registerTab,
  unregisterTab,
  selectTab,
  tabExists,
  teleportId,
};
provide(TabGroupContextInjectionKey, context);

defineExpose({ selectTab, tabExists });
</script>
