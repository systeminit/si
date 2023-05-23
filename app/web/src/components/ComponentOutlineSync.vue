<template>
  <ScrollArea>
    <div class="flex flex-col h-full">
      Diagram Outline
      <div class="grow relative">
        <ComponentOutline />
      </div>
    </div>
  </ScrollArea>
</template>

<script lang="ts" setup>
import { ScrollArea } from "@si/vue-lib/design-system";
import ComponentOutline from "@/components/ComponentOutline/ComponentOutline.vue";
</script>
