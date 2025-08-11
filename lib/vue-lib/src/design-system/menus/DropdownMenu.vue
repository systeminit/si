<template>
  <Teleport to="#app">
    <div
      v-if="isOpen"
      ref="internalRef"
      v-bind="dynamicAttrs"
      :class="
        clsx(
          'z-100 fixed text-sm shadow-[0_4px_8px_0_rgba(0,0,0,0.75)] empty:hidden',
          'flex flex-col',
          {
            editor: 'rounded border border-neutral-600 min-w-[164px]',
            contextmenu: [
              'rounded border',
              themeClasses('border-action-500', 'border-action-300'),
            ],
            classic:
              'outline outline-offset-0 rounded-md outline-neutral-300 outline-0 dark:outline-1',
            compact:
              'outline outline-offset-0 rounded-md outline-neutral-300 outline-0 dark:outline-1',
          }[variant],
          variant === 'contextmenu'
            ? themeClasses(
                'bg-neutral-100 text-shade-100',
                'bg-neutral-800 text-shade-0',
              )
            : 'text-shade-0 bg-shade-100',
          isRepositioning && 'opacity-0',
        )
      "
      :style="computedStyle"
      @mouseenter="setHover"
      @mouseleave="clearHover"
    >
      <SiSearch
        v-if="search"
        ref="siSearchRef"
        class="w-full flex-none"
        variant="dropdownmenu"
        :allFilter="{ name: 'All Views' }"
        :filters="searchFilters"
        @click.stop="selectSearch"
        @blur="delayedDeselectSearch"
        @search="onSearch"
        @clearSearch="onSearch('')"
        @enterPressed="selectFirst"
      />
      <div
        ref="scrollDivRef"
        class="flex-1 overflow-x-hidden overflow-y-auto min-h-[20px]"
      >
        <!-- items can be passed in via props -->
        <DropdownMenuItem
          v-for="item in items"
          ref="menuItemRefs"
          :key="item.label"
          v-bind="item"
          :insideSubmenu="submenu"
        />

        <!-- or use DropdownMenuItem in the default slot -->
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<script lang="ts">
type DropdownMenuContext = {
  variant: DropdownMenuVariant;
  isOpen: Ref<boolean>;
  isCheckable: Ref<boolean>;
  focusedItemId: Ref<string | undefined>;
  search: boolean;
  navigatingSubmenu: Ref<boolean>;

  registerItem(id: string, component: ComponentInternalInstance): void;
  unregisterItem(id: string): void;

  open(e?: MouseEvent, anchorToMouse?: boolean): void;
  close(shouldClose: boolean): void;
  focusOnItem(id?: string): void;
  openSubmenu(id?: string): void;
};

export const DropdownMenuContextInjectionKey: InjectionKey<DropdownMenuContext> =
  Symbol("DropdownMenuContext");

export function useDropdownMenuContext() {
  const ctx = inject(DropdownMenuContextInjectionKey, null);
  if (!ctx)
    throw new Error(
      "<DropdownMenuItem> should only be used within a <DropdownMenu>",
    );
  return ctx;
}
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import {
  ComponentInternalInstance,
  computed,
  inject,
  InjectionKey,
  isRef,
  PropType,
  provide,
  reactive,
  Ref,
  ref,
  unref,
} from "vue";
import DropdownMenuItem from "./DropdownMenuItem.vue";
import { themeClasses, useThemeContainer } from "../utils/theme_tools";
import SiSearch, { Filter } from "../general/SiSearch.vue";

export type DropdownMenuItemObjectDef = InstanceType<
  typeof DropdownMenuItem
>["$props"];

export type DropdownMenuVariant =
  | "classic"
  | "compact"
  | "editor"
  | "contextmenu";

const MENU_EDGE_BUFFER = 10;

// For Submenus, the anchorTo prop holds an object with this info -
interface SubmenuParent {
  $el: Element;
  close: (shouldNotClose?: boolean, closeRecursively?: boolean) => void;
  navigatingSubmenu: Ref<boolean>;
}

// IMPORTANT NOTE - currently any DropdownMenu with a dynamic number of DropdownMenuItems cannot have submenus
const props = defineProps({
  // Set an anchorTo element if you want the DropdownMenu to be attached to a DOM element
  // If no anchorTo element is used, each open() event for this Dropdown will try to determine where to anchor based on the mouse position or event target
  anchorTo: { type: Object || HTMLElement },

  // You can add DropdownMenuItems via this prop or in a template
  items: {
    type: Array as PropType<DropdownMenuItemObjectDef[]>,
  },
  // Each variant has slightly different styles
  variant: {
    type: String as PropType<DropdownMenuVariant>,
    default: "compact",
  },
  // Turn this boolean on to prevent the default closing behavior and only close when told to externally
  // The menu will still close if you scroll or resize the document
  noDefaultClose: Boolean,
  disableKeyboardControls: Boolean, // disable the keyboard controls for this DropdownMenu

  // Alignment properties to adjust how the menu behaves in terms of position/alignment
  forceAbove: Boolean, // forces the menu to appear above the anchor position
  forceAlignRight: Boolean, // forces the menu to align to the right edge of the anchor position instead of defaulting to aligning left
  alignCenter: Boolean, // aligns the menu to be centered on the anchor position horizontally
  alignOutsideRightEdge: Boolean, // aligns the menu's left edge with the right edge of the anchor element
  alignOutsideLeftEdge: Boolean, // aligns the menu's right edge with the left edge of the anchor element
  overlapAnchorOnAnchorTo: Boolean, // adjusts the menu position to cover the anchor element instead of positioning on its edge
  overlapAnchorOffset: { type: Number, default: 0 }, // adjust the overlap position with a fixed number
  anchorXOffset: { type: Number, default: 0 }, // adjust the position of the menu on the x axis with a fixed number

  // SUBMENUS CAN BREAK BE AWARE OF HOW YOU USE THEM WITH A DYNAMIC NUMBER OF CHILD ELEMENTS
  submenu: Boolean, // If this is a submenu, the parent menu element is in the anchorTo prop!

  // Props for a search bar at the top of this DropdownMenu
  search: Boolean,
  searchFilters: Array<Filter>,

  maxWidth: { type: Number, default: 280 }, // change this to adjust the maximum width of the DropdownMenu
  matchWidthToAnchor: { type: Boolean }, // forces the width of the menu to match the anchorTo element's width
  minWidthToAnchor: { type: Boolean }, // forces the width of the menu to match or be bigger than the anchorTo element's width
});

useThemeContainer(props.variant !== "contextmenu" ? "dark" : undefined);

const internalRef = ref<HTMLElement | null>(null);
const scrollDivRef = ref();
const siSearchRef = ref<InstanceType<typeof SiSearch>>();

function nextFrame(cb: () => void) {
  requestAnimationFrame(() => requestAnimationFrame(cb));
}

// Items, registration, settings /////////////////////////////////////////////////////////////////
const itemsById = reactive({} as Record<string, ComponentInternalInstance>);
const sortedItemIds = ref<string[]>([]);
const focusedItemId = ref<string>();
const isCheckable = ref(false);

function registerItem(id: string, component: ComponentInternalInstance) {
  itemsById[id] = component;
  refreshSortedItemIds();
  refreshSettingsFromItems();
}
function unregisterItem(id: string) {
  delete itemsById[id];
  refreshSortedItemIds();
  refreshSettingsFromItems();
}

function refreshSortedItemIds() {
  if (!isOpen.value) return;
  sortedItemIds.value = Object.keys(itemsById).sort((id1, id2) => {
    // TODO: extract this logic into utility which we can reuse
    let domNode1 = itemsById[id1]?.exposed?.domRef;
    let domNode2 = itemsById[id2]?.exposed?.domRef;
    if (isRef(domNode1)) domNode1 = domNode1.value;
    if (isRef(domNode2)) domNode2 = domNode2.value;
    if (domNode1.$el) domNode1 = domNode1.$el;
    if (domNode2.$el) domNode2 = domNode2.$el;
    if (!domNode1 || !domNode2) return 0;
    const position = domNode1.compareDocumentPosition(domNode2);
    /* eslint-disable no-bitwise */
    if (position & Node.DOCUMENT_POSITION_FOLLOWING) return -1;
    if (position & Node.DOCUMENT_POSITION_PRECEDING) return 1;
    return 0;
  });
}

// some settings come from the children
// ex: the menu being "checkable" is based on if any children have checkable set
function refreshSettingsFromItems() {
  isCheckable.value = _.some(itemsById, (item) => !!item.props?.checkable);
}

// Focused item management //////////////////////////////////////////////////////////////////////////////

const focusedItemIndex = computed({
  get() {
    if (!focusedItemId.value) return undefined;
    return sortedItemIds.value.indexOf(focusedItemId.value);
  },
  set(newIndex: number | undefined) {
    if (newIndex === undefined) {
      focusedItemId.value = undefined;
      return;
    }
    let validIndex = newIndex;
    if (validIndex < 0) validIndex = 0;
    else if (validIndex >= sortedItemIds.value.length)
      validIndex = sortedItemIds.value.length - 1;
    focusedItemId.value = sortedItemIds.value[validIndex];
  },
});
const focusedItem = computed(() => {
  if (!focusedItemId.value) return;
  return itemsById[focusedItemId.value];
});
const focusedItemEl = computed(() => {
  // some weird behaviour where things can be inconsistently wrapped in a ref...
  // TODO: figure this out and make some utility fns
  const domRef = unref(focusedItem.value?.exposed?.domRef);
  const el = domRef?.$el || domRef;
  return el;
});

function focusOnItem(id?: string) {
  if (id && itemsById[id]) focusedItemId.value = id;
  else focusedItemId.value = undefined;

  if (id) {
    navigatingSubmenu.value = false;
    if (props.submenu && props.anchorTo) {
      const parent = props.anchorTo as SubmenuParent;
      parent.navigatingSubmenu.value = true;
    }
  }
}

// Submenu navigation booleans
const navigatingSubmenu = ref(false);
const navigatingParent = computed(() => {
  if (props.submenu && props.anchorTo) {
    const parent = props.anchorTo as SubmenuParent;
    if (!parent.navigatingSubmenu.value) {
      return true;
    }
  }
  return false;
});

// Opening / closing / positioning ////////////////////////////////////////////////////////////////////
const isOpen = ref(false);
const readOnlyIsOpen = computed(() => isOpen.value);
const isRepositioning = ref(false);

function open(e?: MouseEvent, anchorToMouse?: boolean) {
  const clickTargetIsElement =
    e?.target instanceof HTMLElement || e?.target instanceof Element;

  if (props.anchorTo) {
    // can anchor to a specific element via props
    if (props.anchorTo instanceof HTMLElement) {
      anchorEl.value = props.anchorTo;
    } else {
      anchorEl.value = props.anchorTo.$el;
    }
  } else if (e && (anchorToMouse || !clickTargetIsElement)) {
    // or can anchor to mouse position if anchorToMouse is true (or event has not target)
    anchorEl.value = undefined;
    anchorPos.value = { x: e?.clientX, y: e.clientY };
  } else if (clickTargetIsElement) {
    // otherwise anchor to click event target
    anchorEl.value = e.target;
  } else {
    // shouldn't happen...?
    anchorEl.value = undefined;
  }

  isRepositioning.value = true;
  isOpen.value = true;
  navigatingSubmenu.value = false;

  focusOnItem();
  nextFrame(finishOpening);
}
function finishOpening() {
  readjustMenuPosition();
  startListening();
  if (props.search && siSearchRef.value) {
    siSearchRef.value.focusSearch();
    selectSearch();
  }
}
function close(
  shouldNotClose = false,
  closeRecursively = true,
  forceClose = false,
) {
  if (shouldNotClose) return;
  if (props.noDefaultClose && closeRecursively && !forceClose) return;
  isOpen.value = false;
  if (oneTimeCloseListener.value) {
    window.removeEventListener("click", oneTimeCloseListener.value);
    oneTimeCloseListener.value = undefined;
  }
  stopListening();
  clearPositioningData();
  emit("onClose");
  if (
    props.submenu &&
    props.anchorTo &&
    props.anchorTo.close &&
    closeRecursively
  ) {
    (props.anchorTo as SubmenuParent).close();
  }
}
function forceClose() {
  close(false, true, true);
}
function closeOnResizeOrScroll(e: Event) {
  // because a scroll event in the DiagramOutline can be a side effect of opening the editor right click menu
  // this behavior is disabled for the editor variant for scroll events
  if (
    scrollDivRef.value &&
    scrollDivRef.value !== e.target &&
    !(props.variant === "editor" && e.type === "scroll")
  ) {
    close();
  }
}
function clearPositioningData() {
  anchorEl.value = undefined;
  anchorPos.value = undefined;
  menuHeight.value = undefined;
  hAlign.value = "left";
  vAlign.value = "below";
}

const anchorEl = ref<HTMLElement | Element>();
const anchorPos = ref<{ x: number; y: number }>();

const menuHeight = ref<number | undefined>(undefined);
const hAlign = ref<"left" | "right">("left");
const vAlign = ref<"below" | "above">("below");
const posX = ref(0);
const posY = ref(0);

function readjustMenuPosition() {
  if (!internalRef.value) return;

  menuHeight.value = undefined;
  isRepositioning.value = false;

  let anchorRect: DOMRect;
  if (anchorEl.value) {
    anchorRect = anchorEl.value.getBoundingClientRect();
  } else if (anchorPos.value) {
    anchorRect = new DOMRect(anchorPos.value.x, anchorPos.value.y);
  } else {
    throw new Error("Menu must be anchored to an element or mouse position");
  }
  const menuRect = internalRef.value.getBoundingClientRect();

  // try positioning the menu aligned left with the anchor, and if goes off screen align right with end of screen
  hAlign.value = "left";
  posX.value = anchorRect.x;
  if (props.submenu) {
    posX.value = anchorRect.right;
  } else if (props.alignCenter) {
    posX.value = anchorRect.x + anchorRect.width / 2 - menuRect.width / 2;
  } else if (props.alignOutsideRightEdge) {
    posX.value = anchorRect.x + anchorRect.width;
  } else if (props.alignOutsideLeftEdge) {
    posX.value = anchorRect.x - menuRect.width;
  }
  // NOTE - window.innerWidth was including scrollbar width, so throwing off calc
  const windowWidth = document.documentElement.clientWidth;
  if (props.forceAlignRight) {
    hAlign.value = "right";
    posX.value = windowWidth - anchorRect.right;
  } else if (posX.value + menuRect.width > windowWidth) {
    hAlign.value = "right";
    posX.value = 4; // if overflowing off the screen, we right align with a small buffer
    if (props.submenu) {
      hAlign.value = "left";
      posX.value = anchorRect.left - menuRect.width - 4;
    }
  }
  posX.value += props.anchorXOffset;

  const overlapOffset = anchorRect.height + props.overlapAnchorOffset;

  // try positioning the menu below the anchor
  const positionBelow = () => {
    vAlign.value = "below";
    posY.value = anchorRect.bottom + 4;
    if (props.submenu) {
      posY.value = anchorRect.top;
    } else if (
      props.overlapAnchorOnAnchorTo ||
      props.alignOutsideRightEdge ||
      props.alignOutsideLeftEdge
    ) {
      posY.value -= overlapOffset;
    }
  };
  positionBelow();
  const availableHeightBelow =
    window.innerHeight - posY.value - MENU_EDGE_BUFFER;

  // if the menu does not fit below the anchor or if forceAbove is enabled, position it above the anchor
  if (props.forceAbove || posY.value + menuRect.height > window.innerHeight) {
    vAlign.value = "above";
    posY.value = window.innerHeight - (anchorRect.top - 4);
    if (
      props.overlapAnchorOnAnchorTo ||
      props.alignOutsideRightEdge ||
      props.alignOutsideLeftEdge
    ) {
      posY.value -= overlapOffset;
    }
    const availableHeightAbove =
      window.innerHeight - posY.value - MENU_EDGE_BUFFER;

    // Check if the menu goes off the top of the screen
    if (window.innerHeight - posY.value - menuRect.height < 0) {
      // The menu does not fit above or below the anchor position so we need to constrain the menu height and enable scrolling
      if (props.forceAbove || availableHeightAbove > availableHeightBelow) {
        // constrain the height of the menu and put it above
        menuHeight.value = window.innerHeight - posY.value - MENU_EDGE_BUFFER;
      } else {
        // constrain the height of the menu and put it below
        positionBelow();
        menuHeight.value = window.innerHeight - posY.value - MENU_EDGE_BUFFER;
        if (
          props.overlapAnchorOnAnchorTo ||
          props.alignOutsideRightEdge ||
          props.alignOutsideLeftEdge
        ) {
          menuHeight.value -= overlapOffset;
        }
      }
    }
  }
}

const APP_MINIMUM_WIDTH = 700; // APP_MINIMUM_WIDTH
const getWindowWidth = () => {
  if (window.innerWidth > APP_MINIMUM_WIDTH) return window.innerWidth;
  else return APP_MINIMUM_WIDTH;
};

// eslint-disable-next-line @typescript-eslint/ban-types
const computedStyle: Object = computed(() => ({
  ...(hAlign.value === "left" && { left: `${posX.value}px` }),
  ...(hAlign.value === "right" && { right: `${posX.value}px` }),
  ...(vAlign.value === "below" && { top: `${posY.value}px` }),
  ...(vAlign.value === "above" && { bottom: `${posY.value}px` }),
  ...(menuHeight.value && { maxHeight: `${menuHeight.value}px` }),
  ...(props.matchWidthToAnchor &&
    anchorEl.value && {
      width: `${anchorEl.value.getBoundingClientRect().width}px`,
    }),
  ...(props.minWidthToAnchor &&
    anchorEl.value &&
    (props.forceAlignRight
      ? {
          minWidth: `${anchorEl.value.getBoundingClientRect().width}px`,
          maxWidth: `${Math.min(
            anchorEl.value.getBoundingClientRect().right - MENU_EDGE_BUFFER,
            getWindowWidth() / 2,
          )}px`, // the maximum width of a dropdown menu with this setting is half of the browser window width
        }
      : {
          minWidth: `${anchorEl.value.getBoundingClientRect().width}px`,
          maxWidth: `${Math.min(
            getWindowWidth() -
              anchorEl.value.getBoundingClientRect().left -
              MENU_EDGE_BUFFER,
            getWindowWidth() / 2,
          )}px`, // the maximum width of a dropdown menu with this setting is half of the browser window width
        })),
  ...(!props.matchWidthToAnchor &&
    !props.minWidthToAnchor &&
    anchorEl.value && { maxWidth: `${props.maxWidth}px` }),
}));

// Event handling //////////////////////////////////////////////////////////////////////////////////////////////

function startListening() {
  if (!props.disableKeyboardControls) {
    window.addEventListener("keydown", onKeyboardEvent);
  }
  window.addEventListener("mousedown", onWindowMousedown);
  window.addEventListener("resize", closeOnResizeOrScroll);
  window.addEventListener("scroll", closeOnResizeOrScroll, true);
}

const oneTimeCloseListener = ref<undefined | (() => void)>(undefined);
const createOneTimeCloseListener = (noCloseOnClick: boolean) => {
  return () => {
    close(noCloseOnClick);
  };
};
function onWindowMousedown(e: MouseEvent) {
  if (
    e.target &&
    e.target instanceof Element &&
    e.target.closest(".siSearchRoot")
  ) {
    // do not close the Dropdown if you click on the search bar!
    return;
  }

  if (
    e.target instanceof Element &&
    e.target.getAttribute("role") === "menuitem"
  ) {
    // do not close if the item clicked is in a submenu, allow the submenu to handle the click and whether or not to close
    return;
  } else if (
    e.target instanceof Element &&
    internalRef.value?.contains(e.target) &&
    !oneTimeCloseListener.value
  ) {
    // then detect clicks on one of this menu's children and respond accordingly
    const noCloseOnClick = Boolean(
      e.target.getAttribute("data-no-close-on-click"),
    );
    oneTimeCloseListener.value = createOneTimeCloseListener(noCloseOnClick);
    window.addEventListener("click", oneTimeCloseListener.value, {
      once: true,
    });
  } else if (!(props.submenu && e.target === props.anchorTo?.$el)) {
    // finally, close this menu unless it is a submenu and the element being clicked is the parent
    close(props.noDefaultClose);
  }
}
function closeFocusedSubmenu() {
  if (focusedItem.value?.exposed) {
    focusedItem.value.exposed.closeSubmenu();
  }
}
function onKeyboardEvent(e: KeyboardEvent) {
  if (!navigatingSubmenu.value && !navigatingParent.value) {
    if (e.key === "ArrowUp") {
      closeFocusedSubmenu();
      if (focusedItem.value?.exposed) {
        focusedItem.value.exposed.closeSubmenu();
      }
      if (focusedItemIndex.value === undefined)
        focusedItemIndex.value = sortedItemIds.value.length - 1;
      else focusedItemIndex.value -= 1;
      e.preventDefault();
      focusedItem.value?.exposed?.openSubmenu();
    } else if (e.key === "ArrowDown") {
      closeFocusedSubmenu();
      if (focusedItemIndex.value === undefined) focusedItemIndex.value = 0;
      else focusedItemIndex.value += 1;
      e.preventDefault();
      focusedItem.value?.exposed?.openSubmenu();
    }
  }

  if (
    e.key === "ArrowRight" &&
    focusedItem &&
    focusedItem.value?.exposed?.hasSubmenu
  ) {
    navigatingSubmenu.value = true;
    focusedItem.value.exposed.focusFirstSubmenuItem();
  } else if (e.key === "ArrowLeft" && props.submenu && props.anchorTo) {
    const parent = props.anchorTo as SubmenuParent;
    parent.navigatingSubmenu.value = false;
    closeFocusedSubmenu();
    focusOnItem();
  }

  if (searchSelected.value) {
    if (e.key === "Escape") {
      deselectSearch();
    }
  } else if (e.key === "Enter" || e.key === " ") {
    if (focusedItemEl.value) {
      // TODO(WENDY) - how does this part conflict with using the search bar?
      focusedItemEl.value.click();
      e.preventDefault();
    } else {
      emit("enterPressedNoSelection");
    }
  } else if (e.key === "Escape") {
    close();
  }
}
function stopListening() {
  window.removeEventListener("keydown", onKeyboardEvent);
  window.removeEventListener("mousedown", onWindowMousedown);
  window.removeEventListener("resize", closeOnResizeOrScroll);
  window.addEventListener("scroll", closeOnResizeOrScroll, true);
}

// additional attributes bound onto the root node - used for accessibility attributes
const dynamicAttrs = computed(() => ({
  tabindex: 0,
  "aria-activedescendant": focusedItemId.value || undefined,
  // TODO: if we know it is anchored to an element, we could set this here (if an id exists)
  // 'aria-labelledby': dom(api.buttonRef)?.id,
}));

// handling submenus
function openSubmenu(id?: string) {
  Object.values(itemsById).forEach((item) => {
    if (item.refs.submenuRef) {
      (item.refs.submenuRef as SubmenuParent).close(false, false);
    }
  });
}

// Externally exposed info /////////////////////////////////////////////////////////////////////////////////////////

// this object gets provided to the child DropDownMenuItems
const context = {
  isOpen: readOnlyIsOpen,
  isCheckable,
  focusedItemId,
  variant: props.variant,
  open,
  close,
  registerItem,
  unregisterItem,
  focusOnItem,
  openSubmenu,
  search: props.search,
  navigatingSubmenu,
};
provide(DropdownMenuContextInjectionKey, context);

const hovered = ref(false);

const setHover = () => {
  hovered.value = true;
};
const clearHover = () => {
  hovered.value = false;
};

const emit = defineEmits<{
  (e: "search", searchString: string): void;
  (e: "onClose"): void;
  (e: "enterPressedNoSelection"): void;
}>();

function onSearch(searchString: string) {
  emit("search", searchString);
}

function selectFirst() {
  if (focusedItemEl.value) {
    focusedItemEl.value.click();
  }
}

const searchFilteringActive = computed(
  () => siSearchRef.value?.filteringActive,
);
const searchActiveFilters = computed(
  () => siSearchRef.value?.activeFilters || [],
);

const searchSelected = ref(false);
const selectSearch = () => {
  searchSelected.value = true;
};

const delayedDeselectSearch = () => {
  setTimeout(deselectSearch, 500);
};

const deselectSearch = () => {
  if (siSearchRef.value) {
    siSearchRef.value.clearSearch();
  }
  searchSelected.value = false;
};

// check if a given DOM Node is inside this DropdownMenu or any of its submenus
const menuItemRefs = ref<InstanceType<typeof DropdownMenuItem>[]>();
const elementIsInsideMenu = (el: Node) => {
  if (internalRef.value?.contains(el)) return true;

  if (menuItemRefs.value) {
    for (const menuItem of menuItemRefs.value) {
      if (menuItem && menuItem.elementIsInsideSubmenu(el)) return true;
    }
  }

  return false;
};

const focusFirstItem = (onlyIfNoFocus = false) => {
  if (!onlyIfNoFocus || focusedItemIndex.value === undefined) {
    focusedItemIndex.value = 0;
  }
};

// this is what is exposed to the component using this component (via template ref)
defineExpose({
  isOpen: readOnlyIsOpen,
  open,
  close,
  forceClose,
  hovered,
  searchFilteringActive,
  searchActiveFilters,
  elementIsInsideMenu,
  focusFirstItem,
});
</script>

<style lang="less">
h5 {
  margin-top: 0.75em;
  margin-bottom: 0.5em;
  font-weight: bold;
}
h5:first-child {
  margin-top: 0.25em;
}
</style>
