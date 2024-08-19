<template>
  <div class="flex flex-row items-center gap-xs">
    <Icon name="bell" class="text-destructive-600" />
    <div class="flex flex-col text-sm text-center">
      <div>
        SI is currently in maintenance mode. Please try again later. Reach out
        in Discord for more information if this problem persists.
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
</script>
