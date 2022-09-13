<template>
  <SiCollapsible
    :label="`${kind} &quot;${name}&quot;`"
    as="li"
    content-as="ul"
    default-open
    class="w-full"
  >
    <div class="p-4 w-full max-h-96 overflow-hidden">
      <CodeViewer :code="output" border>
        <template #title><HealthIcon status="Ok" /></template>
      </CodeViewer>
    </div>
  </SiCollapsible>
</template>

<script lang="ts" setup>
import HealthIcon from "@/molecules/HealthIcon.vue";
import SiCollapsible from "../SiCollapsible.vue";
import CodeViewer from "../CodeViewer.vue";

defineProps({
  name: { type: String, required: true },
  kind: { type: String, required: true },
  output: { type: String, required: true },
});
</script>
