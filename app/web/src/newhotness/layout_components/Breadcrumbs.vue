<template>
  <ol class="flex flex-row">
    <li v-for="(b, idx) in breadcrumbs" :key="idx">
      <router-link :to="b.url">
        {{ b.name }}
      </router-link>
    </li>
  </ol>
</template>

<script lang="ts" setup>
import { breadcrumbs } from "../logic_composables/navigation_stack";
</script>

<style lang="less" scoped>
ol li:not(:last-child):after {
  content: " > ";
}
</style>
