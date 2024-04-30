<template>
  <div>
    <Collapsible
      v-for="[funcArgName, funcArgViews] in Object.entries(data)"
      :key="funcArgName"
      :defaultOpen="false"
      :label="funcArgName"
      as="ul"
      contentClasses="px-sm"
      extraBorderAtBottomOfContent
      xPadding="double"
    >
      <Collapsible
        v-for="funcView in funcArgViews"
        :key="funcView.valueSourceId"
        :label="funcView.valueSource"
      >
        <DebugViewItem title="Argument Name" :data="funcView.name" />
        <DebugViewItem title="Value Source" :data="funcView.valueSource" />
        <DebugViewItem title="Value Source Id" :data="funcView.valueSourceId" />
        <DebugViewItem title="Value" :data="funcView.value ?? 'NULL/None'" />
        <DebugViewItem
          title="Connection Kind"
          :data="funcView.socketSourceKind ?? 'NULL'"
        />
        <DebugViewItem
          title="Path To Attribute Value"
          :data="funcView.path ?? 'NULL'"
        />
        <DebugViewItem title="Data Is Used" :data="funcView.isUsed" />
      </Collapsible>
    </Collapsible>
  </div>
</template>

<script setup lang="ts">
import { Collapsible } from "@si/vue-lib/design-system";
import { FuncArgDebugView } from "@/store/components.store";
import DebugViewItem from "./DebugViewItem.vue";

defineProps<{ data: { [key: string]: FuncArgDebugView[] } }>();
</script>
