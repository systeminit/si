<template>
  <Modal
    ref="modalRef"
    type="save"
    size="sm"
    saveLabel="Save"
    title="Save Integrations"
    @save="updateIntegrations"
  >
    <VormInput ref="labelRef" v-model="webhookUrl" label="Slack Webhook Url" />
  </Modal>
</template>

<script setup lang="ts">
import { Modal, VormInput } from "@si/vue-lib/design-system";
import { ref, computed } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";

const workspacesStore = useWorkspacesStore();

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
  if (!webhookUrl.value || webhookUrl.value === "" || !integration.value?.pk)
    return;
  workspacesStore.UPDATE_INTEGRATION(integration.value?.pk, webhookUrl.value);

  modalRef.value?.close();
  webhookUrl.value = "";
};

defineExpose({ open });
</script>
