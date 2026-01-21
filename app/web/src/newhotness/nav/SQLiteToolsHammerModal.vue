<template>
  <Modal ref="modalRef" title="Throw">
    <Stack>
      <VormInput v-model="entityKind" label="Entity Kind" type="text" />
      <VormInput v-model="entityId" label="ID" type="text" />
      <NewButton label="Mjolnir!" tone="action" @click="hammer" />
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import * as heimdall from "@/store/realtime/heimdall";
import { Modal, NewButton, Stack, VormInput } from '@si/vue-lib/design-system';
import { EntityKind } from "@/workers/types/entity_kind_types";
import { ref } from "vue";

const props = defineProps<{
  changeSetId: string;
  workspaceId: string;
}>();

const modalRef = ref();
const entityId = ref("");
const entityKind = ref("");

const hammer = () => {
  if (props.workspaceId && props.changeSetId) {
    heimdall.mjolnir(props.workspaceId, props.changeSetId, entityKind.value as EntityKind, entityId.value);
    modalRef.value.close();
  }
};

const open = () => {
  modalRef.value.open();
};

defineExpose({
  open
});
</script>