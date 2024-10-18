<template>
  <Teleport to="#app">
    <div
      v-if="isOpen"
      ref="internalRef"
      v-bind="dynamicAttrs"
      :class="
        clsx(
          'z-100 fixed text-sm shadow-[0_4px_8px_0_rgba(0,0,0,0.75)] empty:hidden text-white bg-black',
          '',
          variant === 'editor'
            ? 'rounded border border-neutral-600 min-w-[164px]'
            : 'p-2xs outline outline-offset-0 rounded-md outline-neutral-300 outline-0 dark:outline-1',
          isRepositioning && 'opacity-0',
        )
      "
      :style="computedStyle"
      @mouseenter="setHover"
      @mouseleave="clearHover"
    >
      <!-- items can be passed in via props -->
      <DropdownMenuItem
        v-for="item in items"
        :key="item.label"
        v-bind="item"
        :insideSubmenu="submenu"
      />

      <!-- or use DropdownMenuItem in the default slot -->
      <slot />
    </div>
  </Teleport>
</template>

<script lang="ts">
type DropdownMenuContext = {
  variant: DropdownMenuVariant;
  isOpen: Ref<boolean>;
  isCheckable: Ref<boolean>;
  focusedItemId: Ref<string | undefined>;

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
import { useThemeContainer } from "../utils/theme_tools";

export type DropdownMenuItemObjectDef = InstanceType<
  typeof DropdownMenuItem
>["$props"];

export type DropdownMenuVariant = "classic" | "compact" | "editor";

useThemeContainer("dark");

// For Submenus, the anchorTo prop holds an object with this info -
interface SubmenuParent {
  $el: Element;
  close: (shouldNotClose?: boolean, closeRecursively?: boolean) => void;
}

const props = defineProps({
  anchorTo: { type: Object }, // TODO: figure out right type to say "template ref / dom element"
  forceAbove: Boolean,
  forceRight: Boolean,
  submenu: Boolean, // If this is a submenu, the parent menu element is in the anchorTo prop!
  forceAlignRight: Boolean,
  items: {
    type: Array as PropType<DropdownMenuItemObjectDef[]>,
  },
  variant: {
    type: String as PropType<DropdownMenuVariant>,
    default: "compact",
  },
  alignCenter: Boolean,
});

const internalRef = ref<HTMLElement | null>(null);

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
  const el = domRef.$el || domRef;
  return el;
});

function focusOnItem(id?: string) {
  if (id && itemsById[id]) focusedItemId.value = id;
  else focusedItemId.value = undefined;
}

// Opening / closing / positioning ////////////////////////////////////////////////////////////////////
const isOpen = ref(false);
const readOnlyIsOpen = computed(() => isOpen.value);
const isRepositioning = ref(false);

function open(e?: MouseEvent, anchorToMouse?: boolean) {
  const clickTargetIsElement =
    e?.target instanceof HTMLElement || e?.target instanceof Element;

  if (props.anchorTo) {
    // can anchor to a specific element via props
    anchorEl.value = props.anchorTo.$el;
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

  focusOnItem();
  nextFrame(finishOpening);
}
function finishOpening() {
  startListening();
  readjustMenuPosition();
}
function close(shouldNotClose = false, closeRecursively = true) {
  if (shouldNotClose) return;
  isOpen.value = false;
  stopListening();
  if (
    props.submenu &&
    props.anchorTo &&
    props.anchorTo.close &&
    closeRecursively
  ) {
    (props.anchorTo as SubmenuParent).close();
  }
  // TODO: could return focus to the menu button (if one exists)
}

const anchorEl = ref<HTMLElement | Element>();
const anchorPos = ref<{ x: number; y: number }>();

const hAlign = ref<"left" | "right">("left");
const vAlign = ref<"below" | "above">("below");
const posX = ref(0);
const posY = ref(0);

function readjustMenuPosition() {
  if (!internalRef.value) return;

  isRepositioning.value = false;

  let anchorRect;
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

  // try positioning the menu below the anchor, and otherwise position above
  vAlign.value = "below";
  posY.value = anchorRect.bottom + 4;
  if (props.submenu) {
    posY.value = anchorRect.top;
  }
  if (props.forceAbove || posY.value + menuRect.height > window.innerHeight) {
    vAlign.value = "above";
    posY.value = window.innerHeight - (anchorRect.top - 4);
  }
}

const computedStyle = computed(() => ({
  ...(hAlign.value === "left" && { left: `${posX.value}px` }),
  ...(hAlign.value === "right" && { right: `${posX.value}px` }),
  ...(vAlign.value === "below" && { top: `${posY.value}px` }),
  ...(vAlign.value === "above" && { bottom: `${posY.value}px` }),
}));

// Event handling //////////////////////////////////////////////////////////////////////////////////////////////

function startListening() {
  window.addEventListener("keydown", onKeyboardEvent);
  window.addEventListener("mousedown", onWindowMousedown);
}
function onWindowMousedown(e: MouseEvent) {
  if (
    e.target instanceof Element &&
    e.target.getAttribute("role") === "menuitem"
  ) {
    // do not close if the item clicked is in a submenu, allow the submenu to handle the click and whether or not to close
    return;
  } else if (
    e.target instanceof Element &&
    internalRef.value?.contains(e.target)
  ) {
    // then detect clicks on one of this menu's children and respond accordingly
    const noCloseOnClick = Boolean(
      e.target.getAttribute("data-no-close-on-click"),
    );
    window.addEventListener(
      "click",
      () => {
        close(noCloseOnClick);
      },
      { once: true },
    );
  } else if (!(props.submenu && e.target === props.anchorTo?.$el)) {
    // finally, close this menu unless it is a submenu and the element being clicked is the parent
    close();
  }
}
function onKeyboardEvent(e: KeyboardEvent) {
  internalRef.value?.focus({ preventScroll: true });

  if (e.key === "ArrowUp") {
    if (focusedItemIndex.value === undefined)
      focusedItemIndex.value = sortedItemIds.value.length - 1;
    else focusedItemIndex.value -= 1;
    // focusedItemEl.value?.focus({ preventScroll: true });
    e.preventDefault();
  } else if (e.key === "ArrowDown") {
    if (focusedItemIndex.value === undefined) focusedItemIndex.value = 0;
    else focusedItemIndex.value += 1;
    e.preventDefault();
  } else if (e.key === "Enter" || e.key === " ") {
    focusedItemEl.value.click();
    e.preventDefault();
  } else if (e.key === "Escape") {
    close();
  }
}
function stopListening() {
  window.removeEventListener("keydown", onKeyboardEvent);
  window.removeEventListener("mousedown", onWindowMousedown);
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
};
provide(DropdownMenuContextInjectionKey, context);

const hovered = ref(false);

const setHover = () => {
  hovered.value = true;
};
const clearHover = () => {
  hovered.value = false;
};

// this is what is exposed to the component usign this component (via template ref)
defineExpose({
  isOpen: readOnlyIsOpen,
  open,
  close,
  hovered,
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
