<template>
  <div :class="clsx('component tile cursor-pointer')">
    <header>
      <Icon name="logo-aws" size="lg" />
      <h2>{{ component.name }}</h2>
      <h3>{{ component.schemaName }}</h3>
      <Icon name="loader" size="lg" />
    </header>
    <ol>
      <li>Qualifications: {{ component.qualificationTotals.total }}</li>
      <li>Resource: {{ component.hasResource }}</li>
      <li>Diff: {{ component.diffCount }}</li>
      <hr />
      <li>Inputs: {{ component.inputCount }}</li>
      <li>Outputs: {{ component.outputCount }}</li>
    </ol>
    <footer class="grid grid-cols-2">
      <div class="place-self-start">
        <VButton label="📌" size="sm" tone="neutral" variant="ghost" />
        <VButton label="Upgrade" size="sm" tone="action" />
      </div>
      <div class="place-self-end">
        <VButton label="Delete" size="sm" tone="destructive" />
        <VButton label="Edit" size="sm" tone="action" />
      </div>
    </footer>
  </div>
</template>

<script lang="ts" setup>
import { Icon, VButton } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";
import { ComponentView } from "@/workers/types/dbinterface";

defineProps({
  component: { type: Object as PropType<ComponentView>, required: true },
});
</script>
