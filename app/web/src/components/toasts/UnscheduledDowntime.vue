<template>
  <div class="flex flex-row items-center gap-xs">
    <Icon name="bell" class="text-destructive-600" />
    <div class="flex flex-col text-sm text-center">
      <div>
        System Initiative was unable to serve the provided request. Please try again later. You can check our
        <a class="text-action-500" href="https://status.systeminit.com">Status Page</a>
        , visit our
        <a class="text-action-500" href="https://discord.com/invite/system-init">Discord</a>
        or email
        <a class="text-action-500" href="mailto:support@systeminit.com">support@systeminit.com</a>
        for information or assistance if this problem persists.
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
</script>
