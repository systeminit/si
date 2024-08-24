<template>
  <RichText class="text-center">
    <h2 class="inline-flex items-center gap-4">
      <Icon class="inline-block" name="alert-circle" />
      Account Suspended
      <Icon class="inline-block" name="alert-circle" />
    </h2>
    <p>
      The account you are using has been suspended pending payment. It cannot
      access System Initiative until this has been rescinded.
      <br />
      <br />
      Contact us at
      <a href="mailto:support@systeminit.com">support@systeminit.com</a> if you
      think this was a mistake.
    </p>
  </RichText>
</template>

<script lang="ts" setup>
import { RichText, Icon } from "@si/vue-lib/design-system";
</script>
