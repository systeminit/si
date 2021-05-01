<template>
  <button
    class="inline-block py-1"
    v-bind:class="buttonStyle"
    :aria-label="label"
    :disabled="disabled"
  >
    <div class="flex justify-center">
      <div class="flex self-center" v-if="icon">
        <PlayIcon :size="iconSize" v-if="icon == 'play'" />
        <SaveIcon :size="iconSize" v-else-if="icon == 'save'" />
        <XIcon :size="iconSize" v-else-if="icon == 'cancel'" />
        <RefreshCcwIcon :size="iconSize" v-else-if="icon == 'refresh'" />
        <EditIcon :size="iconSize" v-else-if="icon == 'edit'" />
        <UploadCloudIcon :size="iconSize" v-else-if="icon == 'deploy'" />
        <LogInIcon :size="iconSize" v-else-if="icon == 'login'" />
        <ArrowUpCircleIcon :size="iconSize" v-else-if="icon == 'signup'" />
        <PlusSquareIcon :size="iconSize" v-else-if="icon == 'plus'" />
        <GitMergeIcon :size="iconSize" v-else-if="icon == 'merge'" />
      </div>
      <div class="ml-1 font-normal" v-if="label && icon != null">
        {{ label }}
      </div>
      <div class="font-normal" v-if="label && !icon">
        {{ label }}
      </div>
    </div>
  </button>
</template>

<script lang="ts">
import Vue from "vue";

import {
  PlusSquareIcon,
  RefreshCcwIcon,
  PlayIcon,
  SaveIcon,
  XIcon,
  EditIcon,
  UploadCloudIcon,
  LogInIcon,
  ArrowUpCircleIcon,
  GitMergeIcon,
} from "vue-feather-icons";

interface ButtonProps {
  kind: "standard" | "save" | "cancel";
  label: null | string;
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
    | "login"
    | "merge";
  size: "xs" | "sm" | "base" | "lg";
  disabled: boolean;
}

export default Vue.extend({
  name: "SiButton",
  components: {
    PlayIcon,
    SaveIcon,
    XIcon,
    RefreshCcwIcon,
    EditIcon,
    UploadCloudIcon,
    LogInIcon,
    ArrowUpCircleIcon,
    PlusSquareIcon,
    GitMergeIcon,
  },
  props: {
    kind: {
      type: String as () => ButtonProps["kind"],
      default: "standard",
    },
    label: {
      type: String as () => ButtonProps["label"],
    },
    icon: {
      type: String as () => ButtonProps["icon"],
    },
    size: {
      type: String as () => ButtonProps["size"],
      default: "base",
    },
    disabled: {
      type: Boolean,
      default: false,
    },
  },
  computed: {
    buttonStyle(): Record<string, boolean> {
      const results: Record<string, boolean> = {};
      results[`button-${this.kind}`] = true;
      results[`text-${this.size}`] = true;
      if (this.disabled) {
        results["opacity-50"] = true;
        results["cursor-not-allowed"] = true;
      }
      return results;
    },
    iconSize(): string {
      switch (this.size) {
        case "xs":
          return "1x";
        case "sm":
          return "1.2x";
        case "base":
          return "1.25x";
        case "lg":
          return "1.5x";
        default:
          return "1.25x";
      }
    },
  },
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
</style>
