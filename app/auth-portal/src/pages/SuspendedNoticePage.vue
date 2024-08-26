<template>
  <RichText class="text-center">
    <h2 class="inline-flex items-center gap-4">
      <Icon class="inline-block" name="alert-circle" />
      Account Paused
      <Icon class="inline-block" name="alert-circle" />
    </h2>
    <p>
      Your account has been paused pending payment or establishing your payment
      information. You can not access System Initiative until this has been
      resolved.
      <br />
      <br />
      Contact
      <a href="mailto:support@systeminit.com">support@systeminit.com</a> if you
      need assistance.
    </p>
  </RichText>
</template>

<script lang="ts" setup>
import { RichText, Icon } from "@si/vue-lib/design-system";
</script>
