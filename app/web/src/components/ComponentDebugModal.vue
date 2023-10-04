<template>
  <Modal ref="modalRef" title="Component Debug" size="4xl">
    <Stack v-if="debugData" class="m-4 overflow-y-scroll max-h-[80vh]">
      <Collapsible label="Component" defaultOpen textSize="lg">
        <dl class="border-l-2 p-2 my-2">
          <dt class="uppercase text-xs italic opacity-80">Id</dt>
          <dd class="p-2 my-2 border-2 border-opacity-10">
            <pre>{{ selectedComponentId ?? "none" }}</pre>
          </dd>
          <dt class="uppercase text-xs italic opacity-80">Name</dt>
          <dd class="p-2 my-2 border-2 border-opacity-10">
            {{ debugData.name }}
          </dd>
          <dt class="uppercase text-xs italic opacity-80">Variant Id</dt>
          <dd class="p-2 my-2 border-2 border-opacity-10">
            <pre>{{ debugData.schemaVariantId }}</pre>
          </dd>
        </dl>
      </Collapsible>
      <Collapsible
        label="Attributes"
        :defaultOpen="false"
        contentAs="ul"
        textSize="lg"
      >
        <Collapsible
          v-for="attribute in debugData.attributes"
          :key="attribute.path"
          class="m-2"
          :label="attribute.path"
          :defaultOpen="false"
          as="li"
        >
          <AttributeDebugView :data="attribute.debugData" />
        </Collapsible>
      </Collapsible>
      <Collapsible
        label="Input Sockets"
        :defaultOpen="false"
        contentAs="ul"
        textSize="lg"
      >
        <Collapsible
          v-for="attribute in debugData.inputSockets"
          :key="attribute.name"
          class="m-2"
          :label="attribute.name"
          :defaultOpen="false"
          as="li"
        >
          <AttributeDebugView :data="attribute.debugData" />
        </Collapsible>
      </Collapsible>
      <Collapsible
        label="Output Sockets"
        :defaultOpen="false"
        contentAs="ul"
        textSize="lg"
      >
        <Collapsible
          v-for="attribute in debugData.outputSockets"
          :key="attribute.name"
          class="m-2"
          :label="attribute.name"
          :defaultOpen="false"
          as="li"
        >
          <AttributeDebugView :data="attribute.debugData" />
        </Collapsible>
      </Collapsible>
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import { Modal, Stack, useModal, Collapsible } from "@si/vue-lib/design-system";
import { ref } from "vue";
import {
  ComponentDebugView,
  useComponentsStore,
} from "@/store/components.store";
import AttributeDebugView from "./AttributeDebugView.vue";

const componentsStore = useComponentsStore();

const modalRef = ref<InstanceType<typeof Modal>>();
const selectedComponentId = ref<string | undefined>();
const debugData = ref<ComponentDebugView | undefined>();

const { open: openModal, close } = useModal(modalRef);

async function open(componentId: string) {
  selectedComponentId.value = componentId;
  const res = await componentsStore.FETCH_COMPONENT_DEBUG_VIEW(componentId);
  if (res.result.success) {
    debugData.value = res.result.data;
    openModal();
  }
}

defineExpose({ open, close });
</script>
