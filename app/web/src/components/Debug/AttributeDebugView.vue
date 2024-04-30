<template>
  <div class="overflow-hidden my-xs p-xs border-opacity-10 border-l-2">
    <dl class="flex flex-col gap-xs">
      <DebugViewItem title="Attribute Value Id" :data="data.attributeValueId" />
      <DebugViewItem :data="data.kind ?? 'any'" title="Type" />
      <DebugViewItem
        :data="`${data.funcName} ${data.funcId}`"
        title="Set By Function"
      />
      <DebugViewItem title="Value" :data="data.value ?? 'NULL'" />
      <DebugViewItem title="Prototype Id" :data="data.prototypeId" />
      <DebugViewItem
        title="Materialized View"
        :data="data.materializedView ?? 'NULL'"
      />
      <DebugViewItem title="Input Sources">
        <template #data>
          <ul v-if="data.funcArgs && Object.keys(data.funcArgs).length">
            <AttributePrototypeDebugView :data="data.funcArgs" />
          </ul>
        </template>
      </DebugViewItem>
    </dl>
  </div>
</template>

<script setup lang="ts">
import { AttributeDebugView } from "@/store/components.store";
import DebugViewItem from "./DebugViewItem.vue";
import AttributePrototypeDebugView from "./AttributePrototypeDebugView.vue";

defineProps<{ data: AttributeDebugView }>();
</script>
