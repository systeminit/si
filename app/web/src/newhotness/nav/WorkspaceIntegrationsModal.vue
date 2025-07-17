<template>
  <Modal
    ref="modalRef"
    type="save"
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
import { ref, computed, inject } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { assertIsDefined, Context } from "../types";

const workspacesStore = useWorkspacesStore();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

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
  if (!webhookUrl.value || webhookUrl.value === "" || !integration.value?.pk)
    return;
  workspacesStore.UPDATE_INTEGRATION(integration.value?.pk, webhookUrl.value);

  modalRef.value?.close();
  webhookUrl.value = "";
};

defineExpose({ open });
</script>
