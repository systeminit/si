<template>
  <IconButton
    icon="dots-vertical"
    variant="classic"
    noBorderOnHover
    iconIdleTone="neutral"
    :selected="selected"
  />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import IconButton from "./IconButton.vue";

defineProps({
  selected: { type: Boolean },
});
</script>
