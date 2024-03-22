<template>
  <div v-if="componentId">
    <template
      v-if="fetchDebugReqStatus.isPending || !fetchDebugReqStatus.isRequested"
    >
      <LoadingMessage>Loading debug details...</LoadingMessage>
    </template>
    <template v-else-if="fetchDebugReqStatus.isError">
      <ErrorMessage :requestStatus="fetchDebugReqStatus" />
    </template>
    <template v-else-if="fetchDebugReqStatus.isSuccess && debugData">
      <div class="border border-neutral-500 m-xs">
        <!-- Component -->
        <Collapsible
          defaultOpen
          extraBorderAtBottomOfContent
          label="Component"
          textSize="lg"
        >
          <dl class="border-l-2 p-xs flex flex-col gap-xs">
            <DebugViewItem :data="componentId" title="Id" />
            <DebugViewItem
              :data="debugData.schemaVariantId"
              title="Variant Id"
            />
          </dl>
        </Collapsible>

        <!-- Attributes -->
        <Collapsible
          :defaultOpen="false"
          contentAs="ul"
          label="Attributes - Not Reimplemented"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.attributes"
            :key="attribute.path"
            :defaultOpen="false"
            :label="attribute.path"
            as="li"
            contentClasses="px-sm"
            extraBorderAtBottomOfContent
            xPadding="double"
          >
            <AttributeDebugView :data="attribute.debugData" />
          </Collapsible>
        </Collapsible>

        <!-- Input Sockets -->
        <Collapsible
          :defaultOpen="false"
          contentAs="ul"
          label="Input Sockets - Not Reimplemented"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.inputSockets"
            :key="attribute.name"
            :defaultOpen="false"
            :label="attribute.name"
            as="li"
            contentClasses="px-sm"
            extraBorderAtBottomOfContent
            xPadding="double"
          >
            <AttributeDebugView :data="attribute.debugData" />
          </Collapsible>
        </Collapsible>

        <!-- Output Sockets -->
        <Collapsible
          :defaultOpen="false"
          contentAs="ul"
          label="Output Sockets - Not Reimplemented"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.outputSockets"
            :key="attribute.name"
            :defaultOpen="false"
            :label="attribute.name"
            as="li"
            contentClasses="px-sm"
            extraBorderAtBottomOfContent
            xPadding="double"
          >
            <AttributeDebugView :data="attribute.debugData" />
          </Collapsible>
        </Collapsible>
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import {
  Collapsible,
  ErrorMessage,
  LoadingMessage,
} from "@si/vue-lib/design-system";
import { PropType, computed, onMounted } from "vue";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import AttributeDebugView from "./AttributeDebugView.vue";
import DebugViewItem from "./DebugViewItem.vue";

const componentsStore = useComponentsStore();

const debugData = computed(
  () => componentsStore.debugDataByComponentId[props.componentId],
);

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const fetchDebugReqStatus = componentsStore.getRequestStatus(
  "FETCH_COMPONENT_DEBUG_VIEW",
  computed(() => props.componentId),
);

onMounted(() => {
  componentsStore.FETCH_COMPONENT_DEBUG_VIEW(props.componentId);
});
</script>
