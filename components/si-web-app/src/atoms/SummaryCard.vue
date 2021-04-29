<template>
  <div class="flex flex-col w-full h-full px-2 py-2 shadow-xl details-panel">
    <div class="w-full h-full details-panel-title">
      <slot name="title" />
    </div>

    <div class="flex w-full h-full ">
      <slot name="content" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "SummaryCard",
});
</script>

<style scoped>
.details-panel {
  border: solid;
  border-width: 1px;
  border-color: #464753;
  background-color: #101010;
}

.details-panel-title {
  /* @apply font-normal text-xs; */
  font-weight: 400;
  font-size: 0.75rem;
  line-height: 1rem;
}
</style>
