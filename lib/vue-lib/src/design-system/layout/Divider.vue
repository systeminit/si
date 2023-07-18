/* General "divider" to draw a line (and optional label) between things */
<template>
  <div class="divider" :style="computedStyles" :class="computedClasses">
    <hr class="divider__line" />
    <div v-if="label" class="divider__label">{{ label }}</div>
    <hr class="divider__line" />
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";

const props = defineProps({
  label: String,
  strong: Boolean,
});

const computedStyles = computed(() => ({}));

const computedClasses = computed(() => ({
  "--strong": props.strong,
}));
</script>

<style lang="less" scoped>
.divider {
  display: flex;
  align-items: center;

  &.--strong {
    .divider__label {
      font-weight: bold;
    }
    .divider__line {
      opacity: 0.9;
    }
  }
}
.divider__label {
  font-size: 12px;
  padding: 0 @spacing-rem[2xs];
  text-transform: uppercase;
}
.divider__line {
  flex: 1 0 0;
  height: 1px;
  border: none;
  background: currentColor;
  opacity: 0.3;
}
</style>
