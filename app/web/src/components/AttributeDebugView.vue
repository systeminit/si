<template>
  <div class="overflow-x-scroll my-2 p-4 border-opacity-10 border-l-2">
    <dl>
      <dt class="uppercase text-xs italic opacity-80">Type</dt>
      <dd class="p-2 my-2 border-2 border-opacity-10">
        <pre>{{ data.kind ?? "any" }}</pre>
      </dd>
      <dt class="uppercase text-xs italic opacity-80">Set By Function</dt>
      <dd class="p-2 my-2 border-2 border-opacity-10">
        <pre>{{ data.funcName }} {{ data.funcId }}</pre>
      </dd>
      <dt class="uppercase text-xs italic opacity-80">Input</dt>
      <dd class="p-2 my-2 border-2 border-opacity-10 overflow-x-scroll">
        <pre>{{ data.funcArgs ?? "NULL" }}</pre>
      </dd>
      <dt class="uppercase text-xs italic opacity-80">Input sources</dt>
      <dd class="p-2 my-2 border-2 border-opacity-10 overflow-x-scroll">
        <ul v-if="data.argSources && Object.keys(data.argSources).length">
          <li v-for="[k, v] in Object.entries(data.argSources)" :key="k">
            <strong>{{ k }}</strong>
            : {{ v ?? "?" }}
          </li>
        </ul>
        <p v-else>No input sources</p>
      </dd>
      <dt class="uppercase text-xs italic opacity-80">Value</dt>
      <dd class="p-2 my-2 border-2 border-opacity-10 overflow-x-scroll">
        <pre>{{ data.value ?? "NULL" }}</pre>
      </dd>
      <dt class="uppercase text-xs italic opacity-80">Prototype Id</dt>
      <dd class="p-2 my-2 border-2 border-opacity-10">
        <pre>{{ data.prototypeId }}</pre>
      </dd>
      <dt class="uppercase text-xs italic opacity-80">Prototype Context</dt>
      <dd class="p-2 my-2 border-2 border-opacity-10">
        <pre>{{ data.prototypeContext }}</pre>
      </dd>
    </dl>
  </div>
</template>

<script setup lang="ts">
import { AttributeDebugData } from "@/store/components.store";

defineProps<{ data: AttributeDebugData }>();
</script>
