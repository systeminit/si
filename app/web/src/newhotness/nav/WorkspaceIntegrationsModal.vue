<template>
  <Modal
    ref="modalRef"
    buttonConfiguration="save"
    size="sm"
    saveLabel="Save"
    title="Save Integrations"
    :disableSave="!currentUserIsDefaultApprover"
    @save="updateIntegrations"
  >
    <VormInput ref="labelRef" v-model="webhookUrl" label="Slack Webhook Url" />
  </Modal>
</template>

<script setup lang="ts">
import { Modal, VormInput } from "@si/vue-lib/design-system";
import { ref, computed } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useContext } from "../logic_composables/context";

const workspacesStore = useWorkspacesStore();

const ctx = useContext();

const currentUserIsDefaultApprover = computed(() =>
  ctx.approvers.value.includes(ctx.user?.pk ?? ""),
);

const modalRef = ref<InstanceType<typeof Modal>>();

const integration = computed(() => workspacesStore.getIntegrations);

function open() {
  if (
    integration.value &&
    integration.value.slackWebhookUrl &&
    integration.value.slackWebhookUrl !== ""
  ) {
    webhookUrl.value = integration.value.slackWebhookUrl;
  }
  modalRef.value?.open();
}

const webhookUrl = ref("");

const updateIntegrations = () => {
  if (!webhookUrl.value || webhookUrl.value === "") return;
  workspacesStore.UPDATE_INTEGRATION(webhookUrl.value);

  modalRef.value?.close();
  webhookUrl.value = "";
};

defineExpose({ open });
</script>
