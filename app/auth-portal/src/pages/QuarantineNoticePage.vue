<template>
  <RichText class="text-center">
    <h2 class="inline-flex items-center gap-4">
      <Icon class="inline-block" name="alert-circle" />
      Account Quarantined
      <Icon class="inline-block" name="alert-circle" />
    </h2>
    <p>
      Your account has been paused due to suspicious activity on your account.
      You can not access System Initiative until this has been resolved.
      <br />
      <br />
      Contact
      <a href="mailto:support@systeminit.com">support@systeminit.com</a> to
      remediate this issue..
    </p>
  </RichText>
</template>

<script lang="ts" setup>
import { RichText, Icon } from "@si/vue-lib/design-system";
</script>
