<template>
  <TabGroupItem closeButton slug="closeButton">
    <template #label>
      <Icon name="x" size="lg" />
    </template>
  </TabGroupItem>
</template>

<script lang="ts" setup>
import Icon from "../icons/Icon.vue";
import TabGroupItem from "./TabGroupItem.vue";
</script>
