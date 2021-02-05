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
      </div>
      <div class="ml-1 font-normal" v-if="label">
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
    | "login";
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

<style scoped>
.button-standard {
  background-color: #50928b;
  @apply text-white px-2;
}

.button-standard:hover {
  background-color: #42a69b;
}

.button-save {
  @apply text-white px-2 bg-green-500;
}

.button-save:hover {
  @apply bg-green-400;
}

.button-cancel {
  @apply text-white px-2 bg-red-500;
}

.button-cancel:hover {
  @apply bg-red-400;
}
</style>
