<template>
  <button
    class="inline-block py-1 h-8 rounded-sm"
    :class="buttonStyle"
    :aria-label="label"
    :disabled="disabled"
  >
    <div class="flex justify-center">
      <div v-if="icon" class="flex self-center">
        <VueFeather v-if="icon === 'play'" type="play" :size="iconSize" />
        <SaveIcon v-else-if="icon === 'save'" :class="heroIconSize" />
        <VueFeather v-else-if="icon === 'cancel'" type="x" :size="iconSize" />
        <VueFeather
          v-else-if="icon === 'refresh'"
          type="refresh-ccw"
          :size="iconSize"
        />
        <VueFeather v-else-if="icon === 'edit'" type="edit" :size="iconSize" />
        <VueFeather
          v-else-if="icon === 'deploy'"
          type="upload-cloud"
          :size="iconSize"
        />
        <VueFeather
          v-else-if="icon === 'login'"
          type="log-in"
          :size="iconSize"
        />
        <VueFeather
          v-else-if="icon === 'signup'"
          type="arrow-up-circle"
          :size="iconSize"
        />
        <VueFeather v-else-if="icon === 'plus'" type="plus" :size="iconSize" />
        <VueFeather
          v-else-if="icon === 'plus-square'"
          type="plus-square"
          :size="iconSize"
        />
        <VueFeather
          v-else-if="icon === 'merge'"
          type="git-merge"
          :size="iconSize"
        />
      </div>
      <div v-if="label && icon != null" class="ml-1 font-normal">
        {{ label }}
      </div>
      <div v-if="label && !icon" class="font-normal">
        {{ label }}
      </div>
    </div>
  </button>
</template>

<script setup lang="ts">
import { computed } from "vue";
import VueFeather from "vue-feather";
import { SaveIcon } from "@heroicons/vue/solid";

interface ButtonProps {
  kind: "standard" | "save" | "cancel";
  label: undefined | string;
  icon:
    | null
    | "play"
    | "save"
    | "cancel"
    | "refresh"
    | "edit"
    | "deploy"
    | "signup"
    | "plus"
    | "plus-square"
    | "login"
    | "merge";
  size: "xs" | "sm" | "base" | "lg";
  disabled: boolean;
}

const props = defineProps({
  kind: {
    type: String as () => ButtonProps["kind"],
    default: "standard",
  },
  label: {
    type: String as () => ButtonProps["label"],
    required: true,
  },
  icon: {
    type: String as () => ButtonProps["icon"],
    default: null,
  },
  size: {
    type: String as () => ButtonProps["size"],
    default: "base",
  },
  disabled: {
    type: Boolean,
    default: false,
  },
});

const buttonStyle = computed(() => {
  const results: Record<string, boolean> = {};
  results[`button-${props.kind}`] = true;
  results[`text-${props.size}`] = true;
  if (props.disabled) {
    results["opacity-50"] = true;
    results["cursor-not-allowed"] = true;
  }
  return results;
});

const iconSize = computed(() => {
  switch (props.size) {
    case "xs":
      return "1rem";
    case "sm":
      return "1.2rem";
    case "base":
      return "1.25rem";
    case "lg":
      return "1.5rem";
    default:
      return "1.25rem";
  }
});

const heroIconSize = computed(() => {
  switch (props.size) {
    case "xs":
      return "w-3 h-3";
    case "sm":
      return "w-4 h-4";
    case "base":
      return "w-5 h-5";
    case "lg":
      return "w-6 h-6";
    default:
      return "w-5 h-5";
  }
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.05;

/* Standard button */
.button-standard {
  // background-color: #50928b;
  background-color: #00b4bc;
  // background-color: #5D9EBE;
  @apply text-white;
  @apply px-2;
}

.button-standard:hover {
  filter: brightness($button-brightness);
}

.button-standard:focus {
  outline: none;
}

.button-standard:active {
  filter: saturate(1.5) brightness($button-brightness);
}

/* Save button */
.button-save {
  @apply text-white;
  @apply px-2;
  @apply bg-green-500;
}

.button-save:hover {
  filter: brightness($button-brightness);
}

.button-save:focus {
  outline: none;
}

.button-save:active {
  filter: saturate($button-saturation) brightness($button-brightness);
}

/* Cancel button */
.button-cancel {
  @apply text-white;
  @apply px-2;
  @apply bg-red-500;
}

.button-cancel:hover {
  filter: brightness($button-brightness);
}

.button-cancel:focus {
  outline: none;
}

.button-cancel:active {
  filter: saturate($button-saturation) brightness($button-brightness);
}
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
