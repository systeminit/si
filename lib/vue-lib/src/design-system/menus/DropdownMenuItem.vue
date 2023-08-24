<template>
  <component
    :is="htmlTagOrComponentType"
    v-bind="dynamicAttrs"
    :id="id"
    ref="internalRef"
    :class="
      clsx(
        'flex gap-xs items-center p-xs pr-sm cursor-pointer rounded-sm children:pointer-events-none',
        isFocused && 'bg-action-500',
        !menuCtx.isCheckable.value && !icon && !$slots.icon && 'pl-sm',
        disabled && 'text-gray-500',
      )
    "
    role="menuitem"
    :tabIndex="disabled === true ? undefined : -1"
    :aria-disabled="disabled === true ? true : undefined"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
    @click="onClick"
  >
    <Icon
      v-if="menuCtx.isCheckable.value"
      :name="checked ? 'check' : 'none'"
      size="xs"
      class="mr-2xs shrink-0"
    />
    <slot name="icon">
      <Icon v-if="icon" :name="icon" size="sm" class="shrink-0" />
    </slot>

    <div ref="labelRef" class="capsize max-w-[220px] shrink-0">
      <div class="truncate">
        <slot>{{ label }}</slot>
      </div>
    </div>
    <div v-if="shortcut" class="pl-md capsize text-xs ml-auto shrink-0">
      {{ shortcut }}
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
  PropType,
  ref,
} from "vue";
import { RouterLink } from "vue-router";
import Icon from "../icons/Icon.vue";
import { IconNames } from "../icons/icon_set";
import { useDropdownMenuContext } from "./DropdownMenu.vue";

const props = defineProps({
  icon: { type: String as PropType<IconNames> },

  label: { type: String },

  // if the item is really a link
  href: String,
  linkToNamedRoute: String,
  linkTo: [String, Object],
  target: String,

  disabled: Boolean,

  // set to true/false only if it is actually checkable - leave undefined otherwise
  // (note the default is needed to not automatically cast to false)
  checked: { type: Boolean, default: undefined },

  shortcut: String,
});

const emit = defineEmits<{ (e: "select"): void }>();

const internalRef = ref<HTMLElement | null>(null);
const menuCtx = useDropdownMenuContext();

const labelText = ref();
const labelRef = ref<HTMLElement>();
const id = `dropdown-menu-item-${idCounter++}`;

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

function onClick(event: MouseEvent) {
  if (props.disabled) {
    event.preventDefault();
    return;
  }
  emit("select");
  menuCtx.close();
}
function onMouseEnter() {
  if (props.disabled) return;
  menuCtx.focusOnItem(id);
}
function onMouseLeave() {
  if (props.disabled) return;
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
  }),

  // set the target when its a link/router link
  ...((htmlTagOrComponentType.value === RouterLink ||
    (htmlTagOrComponentType.value === "a" && props.target)) && {
    target: props.target,
  }),
}));

defineExpose({ domRef: internalRef });
</script>

<script lang="ts">
let idCounter = 1;
</script>
