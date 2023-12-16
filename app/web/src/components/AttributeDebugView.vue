<template>
  <div class="overflow-hidden my-xs p-xs border-opacity-10 border-l-2">
    <dl class="flex flex-col gap-xs">
      <DebugViewItem title="Attribute Value Id" :data="data.valueId" />
      <DebugViewItem :data="data.kind ?? 'any'" title="Type" />
      <DebugViewItem
        :data="`${data.funcName} ${data.funcId}`"
        title="Set By Function"
      />
      <DebugViewItem title="Input" :data="data.funcArgs ?? 'NULL'" />
      <DebugViewItem title="Input sources">
        <template #data>
          <ul v-if="data.argSources && Object.keys(data.argSources).length">
            <li v-for="[k, v] in Object.entries(data.argSources)" :key="k">
              <strong>{{ k }}</strong>
              : {{ v ?? "?" }}
            </li>
          </ul>
          <p v-else>No input sources</p>
        </template>
      </DebugViewItem>
      <DebugViewItem title="Value" :data="data.value ?? 'NULL'" />
      <DebugViewItem title="Prototype Id" :data="data.prototypeId" />
      <DebugViewItem title="Prototype Context" :data="data.prototypeContext" />
      <DebugViewItem
        title="Implicit Attribute Value"
        :data="
          typeof data.implicitValue === 'undefined'
            ? 'none'
            : data.implicitValue ?? 'NULL'
        "
      />
      <DebugViewItem
        title="Implicit Set By Function"
        :data="data.implicitFuncName"
      />
      <p class="text-2xs p-2 my-2 border border-opacity-10">
        prototype in change set?
        {{ data.prototypeInChangeSet ? "y" : "n" }} value in change set?
        {{ data.valueInChangeSet ? "y" : "n" }}
      </p>
    </dl>
  </div>
</template>

<script setup lang="ts">
import { AttributeDebugData } from "@/store/components.store";
import DebugViewItem from "./DebugViewItem.vue";

defineProps<{ data: AttributeDebugData }>();
</script>
