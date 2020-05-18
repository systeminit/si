<template>
  <component
    :is="tag"
    :type="type"
    v-on="$listeners"
    :class="style"
    :kind="kind"
  >
    <slot />
  </component>
</template>

<script>
import { UiButton } from "./UiButtonStyle.js";

export default {
  name: "UiButton",
  props: {
    tag: {
      type: String,
      default: "button",
    },
    type: {
      type: String,
      default: "",
    },
    kind: {
      type: String,
      default: "default",
    },
  },
  computed: {
    style() {
      let theme = new UiButton(this.kind);
      return theme.style;
    },
  },
};
</script>
