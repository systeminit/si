<template>
  <!-- this modal is for the abandoning change sets -->
  <Modal ref="modalRef" title="Abandon Change Set?">
    <div v-if="changeSet.name.length < 50" class="text-md mb-xs">
      Are you sure that you want to abandon change set
      <span class="italic font-bold">
        {{ changeSet?.name }}
      </span>
      and return to HEAD?
    </div>
    <div v-else class="flex flex-col gap-xs text-md mb-xs">
      <div>Are you sure that you want to abandon change set</div>
      <TruncateWithTooltip class="italic font-bold">
        {{ changeSet?.name }}
      </TruncateWithTooltip>
      <div>and return to HEAD?</div>
    </div>
    <div class="text-sm mb-sm">Once abandoned, a change set cannot be recovered.</div>
    <div class="flex flex-row items-center w-full gap-sm">
      <NewButton label="Cancel" tone="warning" icon="x" @click="closeModalHandler" />
      <template v-if="notHead">
        <NewButton
          data-testid="abandon-change-set-modal-confirm-button"
          label="Abandon Change Set"
          tone="destructive"
          class="flex-grow"
          icon="trash"
          loadingText="Abandoning Change Set"
          @click="abandonHandler"
        />
      </template>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { Modal, NewButton, TruncateWithTooltip } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { routes, useApi } from "@/newhotness/api_composables";
import { useContext } from "../logic_composables/context";

const props = defineProps<{
  changeSet: ChangeSet;
}>();

const ctx = useContext();

const notHead = computed(() => ctx.headChangeSetId.value && props.changeSet.id !== ctx.headChangeSetId.value);

const modalRef = ref<InstanceType<typeof Modal> | null>(null);

async function openModalHandler() {
  if (props.changeSet.name === "HEAD" || !notHead.value) return;
  modalRef.value?.open();
}

function closeModalHandler() {
  modalRef.value?.close();
}

const router = useRouter();
const abandonApi = useApi();
async function abandonHandler() {
  const call = abandonApi.endpoint(routes.AbandonChangeSet);
  const { req } = await call.post({ changeSetId: props.changeSet.id });
  if (abandonApi.ok(req)) {
    if (
      router.currentRoute.value.name &&
      ["workspace-lab-packages", "workspace-lab-assets"].includes(router.currentRoute.value.name.toString())
    ) {
      router.push({ name: "workspace-lab" });
    }
  }
  closeModalHandler();
}

defineExpose({ open: openModalHandler });
</script>
