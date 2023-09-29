<template>
  <div class="overflow-x-scroll m-4">
    <dl>
      <dt class="type-bold-base">Set By Function</dt>
      <dd class="m-4 border-black">{{ data.funcName }} {{ data.funcId }}</dd>
      <dt class="type-bold-base">Input</dt>
      <dd class="m-4">
        <pre>{{ data.funcArgs }}</pre>
      </dd>
      <dt class="type-bold-base">Input sources</dt>
      <dd class="m-4">
        <ul v-if="data.argSources && Object.keys(data.argSources).length">
          <li v-for="[k, v] in Object.entries(data.argSources)" :key="k">
            <strong>{{ k }}</strong>
            : {{ v ?? "?" }}
          </li>
        </ul>
        <p v-else>No input sources</p>
      </dd>
      <dt class="type-bold-base">Value</dt>
      <dd class="m-4">
        <pre>{{ data.value }}</pre>
      </dd>
    </dl>
  </div>
</template>

<script setup lang="ts">
import { AttributeDebugData } from "@/store/components.store";

defineProps<{ data: AttributeDebugData }>();
</script>
