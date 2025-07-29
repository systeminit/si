<template>
  <component
    :is="htmlTagOrComponentType"
    :id="id"
    ref="internalRef"
    v-tooltip="tooltip"
    :aria-disabled="noInteract === true ? true : undefined"
    :class="
      clsx(
        'flex gap-xs items-center group select-none',
        sizeClass,
        noInteract ? 'text-neutral-500' : 'cursor-pointer',
        !endLinkTo && !showSecondaryAction && 'children:pointer-events-none',
        header
          ? 'font-bold [&:not(:last-child)]:border-b [&:not(:first-child)]:border-t border-neutral-600'
          : 'rounded-sm',
        isFocused &&
          !header &&
          !menuCtx.navigatingSubmenu.value &&
          themeClasses('bg-action-300', 'bg-action-500'),
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
      :class="clsx('mr-2xs shrink-0 pointer-events-none', checkable && 'ml-xs')"
      size="xs"
    />
    <slot name="icon">
      <Icon
        v-if="icon"
        :class="clsx('shrink-0 pointer-events-none', iconClass ?? '')"
        :name="icon"
        size="sm"
      />
    </slot>

    <div
      v-if="shortcut && menuCtx.variant === 'contextmenu'"
      :class="
        clsx(
          'border rounded px-2xs py-xs min-w-[24px] h-md text-center capsize text-xs',
          themeClasses('border-neutral-400', 'border-neutral-600'),
          shortcutClass ?? '',
        )
      "
    >
      {{ shortcut }}
    </div>

    <div ref="labelRef" class="max-w-full min-w-0 shrink leading-tight">
      <TruncateWithTooltip role="menuitem">
        <slot>{{ label }}</slot>
      </TruncateWithTooltip>
    </div>
    <div
      v-if="!(centerHeader && header)"
      :class="
        clsx(
          'ml-auto shrink-0',
          shortcut && !endLinkTo && !showSecondaryAction && 'capsize text-xs',
          showSecondaryAction && 'h-full',
        )
      "
    >
      <template v-if="submenuItems && submenuItems.length > 0">
        <Icon name="chevron--right" size="sm" />
        <DropdownMenu
          ref="submenuRef"
          :anchorTo="{
            $el: internalRef,
            close: menuCtx.close,
            navigatingSubmenu: menuCtx.navigatingSubmenu,
          }"
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

      <!-- Note(victor) this is rendered transparently when enableSecondaryAction is true,
      so we can get adequate sizes for the items -->
      <div
        v-else-if="enableSecondaryAction"
        :class="
          clsx(
            'h-full flex items-center px-xs',
            themeClasses(
              'group-hover:hover:bg-action-300 group-hover:text-shade-0',
              'group-hover:hover:bg-action-500 group-hover:text-shade-0',
            ),
          )
        "
        @click.stop="onSecondaryActionClick"
        @mouseenter="onMouseEnterEndLink"
        @mouseleave="onMouseLeaveEndLink"
      >
        <Icon
          :name="secondaryActionIcon ?? 'settings-edit'"
          :class="clsx('h-full', !showSecondaryAction && 'text-transparent')"
          size="sm"
        />
      </div>
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
import { tw } from "../../utils/tw-utils";

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
  shortcutClass?: string;
  endLinkLabel?: string;
  endLinkTo?: string | RouteLocationRaw;

  sizeClass?: string;
  enableSecondaryAction?: boolean;
  secondaryActionIcon?: IconNames;

  insideSubmenu?: boolean;
  submenuItems?: DropdownMenuItemProps[];

  submenuVariant?: DropdownMenuVariant;

  // Applies a tooltip to this menu item on the right side
  // Not compatible with a menu item that has submenuItems
  tooltip?: string;
  labelAsTooltip?: boolean;
  showTooltipOnHover?: boolean;
}

const props = defineProps<DropdownMenuItemProps>();

const SUBMENU_TIMEOUT_LENGTH = 300;

useThemeContainer(props.submenuVariant !== "contextmenu" ? "dark" : undefined);

const noInteract = computed(() => props.disabled || props.header);

const emit = defineEmits<{
  (e: "select", event: MouseEvent): void;
  (e: "secondaryAction"): void;
}>();

const internalRef = ref<HTMLElement | null>(null);
const menuCtx = useDropdownMenuContext();

const labelText = ref();
const labelRef = ref<HTMLElement>();
const id = `dropdown-menu-item-${idCounter++}`;

const submenuRef = ref<InstanceType<typeof DropdownMenu> | null>(null);
const submenuTimeout = ref();

const router = useRouter();

const itemIsHovered = ref(false);

const showSecondaryAction = computed(
  () => props.enableSecondaryAction && (itemIsHovered.value || props.checked),
);

// NOTE(nick): I'm so sorry. There was not a clean way to add a new variant that would handle the
// settings edit option without changing sizing in general. Here's the problem: the menu context
// requires the "beforeOptions" and "afterOptions" to follow the variant. We don't always want
// that as some options don't need a settings edit option. I piled another class option on top of
// the class option pile for the component attribute. Please forgive me.
const sizeClass = computed(() => {
  if (props.sizeClass) return props.sizeClass;
  if (props.enableSecondaryAction) return tw`h-[28px]`;
  return {
    classic: tw`p-xs pr-sm`,
    compact: tw`px-xs py-2xs pr-xs`,
    editor: [props.header ? tw`p-xs` : tw`p-2xs pr-xs`, tw`h-7`],
    contextmenu: tw`p-xs pr-sm`,
  }[menuCtx.variant as DropdownMenuVariant];
});

const noCloseOnClick = computed(
  () =>
    props.doNotCloseMenuOnClick ||
    props.header ||
    props.toggleIcon ||
    (props.submenuItems && props.submenuItems.length > 0),
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

function trySelect(event: MouseEvent) {
  if (!noInteract.value) {
    emit("select", event);
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
    trySelect(event);
  } else {
    trySelect(event);
    menuCtx.close(props.doNotCloseMenuOnClick);
  }
}
function onSecondaryActionClick(event: MouseEvent) {
  menuCtx.close(props.doNotCloseMenuOnClick);
  emit("secondaryAction");
}
function onMouseEnter() {
  itemIsHovered.value = true;
  if (noInteract.value) return;
  if (props.submenuItems && props.submenuItems.length > 0) {
    clearTimeout(submenuTimeout.value);
    openSubmenu();
  }
  menuCtx.focusOnItem(id);
}
function onMouseLeave() {
  itemIsHovered.value = false;
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

const hasSubmenu = computed(
  () => props.submenuItems && props.submenuItems.length > 0,
);

const focusFirstSubmenuItem = () => {
  if (submenuRef.value) {
    submenuRef.value.focusFirstItem();
  }
};

defineExpose({
  domRef: internalRef,
  hasSubmenu: hasSubmenu.value,
  elementIsInsideSubmenu,
  openSubmenu,
  closeSubmenu,
  focusFirstSubmenuItem,
});

const tooltip = computed(() => {
  const tooltipTemplate = {
    shown: isFocused.value || (props.showTooltipOnHover && itemIsHovered.value),
    triggers: [],
    placement: "right",
  };

  if (props.labelAsTooltip) {
    return {
      ...tooltipTemplate,
      content: props.label,
    };
  } else if (props.tooltip) {
    return {
      ...tooltipTemplate,
      content: props.tooltip,
    };
  } else return undefined;
});
</script>

<script lang="ts">
let idCounter = 1;
</script>
