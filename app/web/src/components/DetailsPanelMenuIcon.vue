<template>
  <IconButton
    icon="dots-vertical"
    iconIdleTone="neutral"
    :selected="selected"
  />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { IconButton } from "@si/vue-lib/design-system";

defineProps({
  selected: { type: Boolean },
});
</script>
