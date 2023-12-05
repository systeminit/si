<template>
  <div>
    <dt class="uppercase text-xs italic opacity-80">
      <template v-if="title">{{ title }}</template>
      <slot v-else name="title" />
    </dt>
    <dd class="p-2 my-2 border-2 border-opacity-10 overflow-x-auto text-xs">
      <pre><template v-if="data">{{ data }}</template><slot v-else name="data" /></pre>
    </dd>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";

defineProps({
  data: {
    type: [Object, String, Number, Boolean, null] as PropType<
      object | string | number | boolean | null
    >,
  },
  title: { type: String },
});
</script>
