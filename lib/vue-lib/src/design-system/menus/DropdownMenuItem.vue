<template>
  <component
    :is="htmlTagOrComponentType"
    :id="id"
    ref="internalRef"
    :aria-disabled="noInteract === true ? true : undefined"
    :class="
      clsx(
        'flex gap-xs items-center group select-none',
        noInteract ? 'text-neutral-500' : 'cursor-pointer',
        !endLinkTo && 'children:pointer-events-none',
        header
          ? 'font-bold [&:not(:last-child)]:border-b [&:not(:first-child)]:border-t border-neutral-600'
          : 'rounded-sm',
        {
          classic: 'p-xs pr-sm',
          compact: 'px-xs py-2xs pr-xs',
          editor: [header ? 'p-xs' : 'p-2xs pr-xs', 'h-7'],
          contextmenu: 'p-xs pr-sm',
        }[menuCtx.variant as DropdownMenuVariant],
        isFocused && !header && themeClasses('bg-action-300', 'bg-action-500'),
        (!menuCtx.isCheckable.value || disableCheckable) &&
          !icon &&
          !$slots.icon &&
          !header &&
          !toggleIcon &&
          'pl-sm',
        centerHeader && header && 'justify-center',
      )
    "
    :data-no-close-on-click="noCloseOnClick ? true : ''"
    :tabIndex="noInteract === true ? undefined : -1"
    role="menuitem"
    v-bind="dynamicAttrs"
    @click="onClick"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <Toggle
      v-if="toggleIcon"
      :selected="checked || false"
      class="pointer-events-none"
      size="sm"
    />
    <Icon
      v-else-if="menuCtx.isCheckable.value && !disableCheckable"
      :name="checked ? 'check' : 'none'"
      class="mr-2xs shrink-0 pointer-events-none"
      size="xs"
    />
    <slot name="icon">
      <Icon
        v-if="icon"
        :class="
          clsx(
            'shrink-0 pointer-events-none',
            props.iconClass ? props.iconClass : '',
          )
        "
        :name="icon"
        size="sm"
      />
    </slot>

    <div
      v-show="menuCtx.variant !== 'contextmenu' || !icon"
      ref="labelRef"
      class="max-w-full min-w-0 shrink leading-tight"
    >
      <TruncateWithTooltip role="menuitem">
        <slot>{{ label }}</slot>
      </TruncateWithTooltip>
    </div>
    <div
      v-if="shortcut && menuCtx.variant === 'contextmenu'"
      :class="
        clsx(
          'border rounded px-2xs py-xs min-w-[24px] h-md text-center capsize text-xs',
          themeClasses('border-neutral-400', 'border-neutral-600'),
        )
      "
    >
      {{ shortcut }}
    </div>
    <div
      v-else-if="!(centerHeader && header)"
      :class="
        clsx('ml-auto shrink-0', shortcut && !endLinkTo && 'capsize text-xs')
      "
    >
      <template v-if="submenuItems && submenuItems.length > 0">
        <Icon name="chevron--right" size="sm" />
        <DropdownMenu
          ref="submenuRef"
          :anchorTo="{ $el: internalRef, close: menuCtx.close }"
          :items="submenuItems"
          submenu
          :variant="submenuVariant ?? 'editor'"
        />
      </template>

      <div
        v-else-if="endLinkTo"
        :class="
          clsx(
            themeClasses(
              'text-action-500 group-hover:hover:text-action-500 group-hover:text-shade-0',
              'text-action-300 group-hover:hover:text-action-300 group-hover:text-shade-0',
            ),
            'font-bold hover:underline',
          )
        "
        @mousedown="onClickEndLink"
        @mouseenter="onMouseEnterEndLink"
        @mouseleave="onMouseLeaveEndLink"
      >
        <slot name="endLinkLabel">
          <div v-if="endLinkLabel">{{ endLinkLabel }}</div>
          <Icon v-else name="link" />
        </slot>
      </div>
      <template v-else-if="shortcut">
        {{ shortcut }}
      </template>
    </div>
  </component>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import {
  computed,
  getCurrentInstance,
  onBeforeUnmount,
  onMounted,
  ref,
} from "vue";
import { RouteLocationRaw, RouterLink, useRouter } from "vue-router";
import Icon from "../icons/Icon.vue";
import { IconNames } from "../icons/icon_set";
import DropdownMenu, {
  DropdownMenuVariant,
  useDropdownMenuContext,
} from "./DropdownMenu.vue";
import TruncateWithTooltip from "../general/TruncateWithTooltip.vue";
import Toggle from "../general/Toggle.vue";
import { themeClasses, useThemeContainer } from "../utils/theme_tools";

export interface DropdownMenuItemProps {
  icon?: IconNames;
  iconClass?: string;
  toggleIcon?: boolean;

  label?: string;

  // if the item is really a link
  href?: string;
  linkToNamedRoute?: string;
  linkTo?: [string, object];
  target?: string;

  header?: boolean;
  centerHeader?: boolean;
  disabled?: boolean;

  checkable?: boolean;
  checked?: boolean;
  // set this particular DropdownMenuItem to not have a checkmark in a list of otherwise checkable items
  disableCheckable?: boolean;

  doNotCloseMenuOnClick?: boolean;

  shortcut?: string;
  endLinkLabel?: string;
  endLinkTo?: string | RouteLocationRaw;

  insideSubmenu?: boolean;
  submenuItems?: DropdownMenuItemProps[];

  submenuVariant?: DropdownMenuVariant;
}

const props = defineProps<DropdownMenuItemProps>();

const SUBMENU_TIMEOUT_LENGTH = 300;

useThemeContainer(props.submenuVariant !== "contextmenu" ? "dark" : undefined);

const noInteract = computed(() => props.disabled || props.header);

const emit = defineEmits<{ (e: "select"): void }>();

const internalRef = ref<HTMLElement | null>(null);
const menuCtx = useDropdownMenuContext();

const labelText = ref();
const labelRef = ref<HTMLElement>();
const id = `dropdown-menu-item-${idCounter++}`;

const submenuRef = ref<InstanceType<typeof DropdownMenu> | null>(null);
const submenuTimeout = ref();

const router = useRouter();

const noCloseOnClick = computed(
  () =>
    !!(
      props.doNotCloseMenuOnClick ||
      props.header ||
      props.toggleIcon ||
      (props.submenuItems && props.submenuItems.length > 0)
    ),
);

onMounted(() => {
  // track text in label to be used for typing to jump to an option
  labelText.value = labelRef.value?.textContent?.toLowerCase().trim();

  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  menuCtx.registerItem(id, getCurrentInstance()!);
});
onBeforeUnmount(() => {
  menuCtx.unregisterItem(id);
});

const htmlTagOrComponentType = computed(() => {
  if (props.href) return "a";
  if (props.linkTo || props.linkToNamedRoute) return RouterLink;
  return "div";
});

const isFocused = computed(() => {
  return menuCtx.focusedItemId.value === id;
});

function trySelect() {
  if (!noInteract.value) {
    emit("select");
  }
}

function onClick(event: MouseEvent) {
  if (
    noCloseOnClick.value ||
    (props.submenuItems && props.submenuItems.length > 0)
  ) {
    event.preventDefault();
    if (
      props.submenuItems &&
      props.submenuItems.length > 0 &&
      !submenuRef.value?.isOpen
    ) {
      openSubmenu();
    }
    trySelect();
  } else {
    trySelect();
    menuCtx.close(props.doNotCloseMenuOnClick);
  }
}
function onMouseEnter() {
  if (noInteract.value) return;
  if (props.submenuItems && props.submenuItems.length > 0) {
    clearTimeout(submenuTimeout.value);
    openSubmenu();
  }
  menuCtx.focusOnItem(id);
}
function onMouseLeave() {
  if (noInteract.value) return;
  if (
    props.submenuItems &&
    props.submenuItems.length > 0 &&
    !submenuRef.value?.hovered
  ) {
    submenuTimeout.value = setTimeout(() => {
      if (!submenuRef.value?.hovered) {
        closeSubmenu();
      }
    }, SUBMENU_TIMEOUT_LENGTH);
  }
  if (!isFocused.value) return;
  menuCtx.focusOnItem();
}

// some attributes need to get set only if the item is a router link or <a>
// similar logic to VButton - maybe can DRY this up at some point
const dynamicAttrs = computed(() => ({
  // set the "to" prop if we are in router link mode
  ...(htmlTagOrComponentType.value === RouterLink && {
    to: props.linkToNamedRoute
      ? { name: props.linkToNamedRoute }
      : props.linkTo,
  }),

  // if we set href to undefined when in RouterLink mode, it doesn't set it properly
  ...(htmlTagOrComponentType.value === "a" && {
    href: props.href,
    target: "_blank",
  }),

  // set the target when its a link/router link
  ...((htmlTagOrComponentType.value === RouterLink ||
    (htmlTagOrComponentType.value === "a" && props.target)) && {
    target: props.target,
  }),
}));

const openSubmenu = () => {
  if (submenuRef.value && props.submenuItems && props.submenuItems.length > 0) {
    menuCtx.openSubmenu(id);
    submenuRef.value.open();
  }
};

const closeSubmenu = () => {
  if (submenuRef.value) submenuRef.value.close(false, false);
};

function onMouseEnterEndLink() {
  menuCtx.focusOnItem();
}

function onMouseLeaveEndLink() {
  menuCtx.focusOnItem(id);
}

async function onClickEndLink() {
  if (props.endLinkTo) {
    await router.push(props.endLinkTo);
  }
}

const elementIsInsideSubmenu = (el: Node) => {
  if (submenuRef.value) return submenuRef.value.elementIsInsideMenu(el);
  else return false;
};

defineExpose({ domRef: internalRef, elementIsInsideSubmenu });
</script>

<script lang="ts">
let idCounter = 1;
</script>
