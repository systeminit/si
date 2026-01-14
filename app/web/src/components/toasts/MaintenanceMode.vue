<template>
  <div class="flex flex-row items-center gap-xs">
    <Icon name="bell" class="text-destructive-600" />
    <div class="flex flex-col text-sm text-center">
      <div>
        System Initiative is currently in maintenance mode. Please try again later. You can check
        <a class="text-action-500" href="https://discord.com/invite/system-init">Discord</a>
        for more information or to get assistance. You can also email
        <a class="text-action-500" href="mailto:support@systeminit.com">support@systeminit.com</a>
        if this problem persists.
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
</script>
