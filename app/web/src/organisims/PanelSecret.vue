<template>
  <Panel
    :panel-index="props.panelIndex"
    :panel-ref="props.panelRef"
    :panel-container-ref="props.panelContainerRef"
    :initial-maximized-container="props.initialMaximizedContainer"
    :initial-maximized-full="props.initialMaximizedFull"
    :is-visible="props.isVisible"
    :is-maximized-container-enabled="props.isMaximizedContainerEnabled"
  >
    <template #menuButtons>
      <div class="justify-start flew flex-grow">
        <div class="min-w-max pl-2 align-middle">
          <SiButton
            icon="plus"
            label="new"
            size="xs"
            class="pl-2 align-middle"
            :disabled="!enableNewSecretButton"
            @click="newSecretClick"
          />
        </div>
      </div>
    </template>
    <template #content>
      <div class="w-full">
        <SecretList v-if="renderView == 'list'" />
        <div
          v-else-if="renderView == 'create'"
          class="flex items-center justify-center"
        >
          <div
            class="flex flex-grow px-4 py-4 mx-8 mt-8 border border-gray-700"
          >
            <SecretCreate
              @cancel="secretCreateCancel"
              @submit="secretCreateSubmit"
            />
          </div>
        </div>
      </div>
    </template>
  </Panel>
</template>

<script setup lang="ts">
import SecretCreate from "@/organisims/Secret/SecretCreate.vue";
import SecretList from "@/organisims/Secret/SecretList.vue";
import Panel from "@/molecules/Panel.vue";
import SiButton from "@/atoms/SiButton.vue";
import { computed, ref } from "vue";
import { refFrom } from "vuse-rx";
import { ChangeSetService } from "@/service/change_set";

const props = defineProps<{
  panelIndex: number;
  panelRef: string;
  panelContainerRef: string;
  initialMaximizedFull?: boolean;
  initialMaximizedContainer?: boolean;
  isVisible?: boolean;
  isMaximizedContainerEnabled?: boolean;
}>();

const editMode = refFrom<boolean | undefined>(
  ChangeSetService.currentEditMode(),
);

const renderView = ref<"create" | "list">("list");

const newSecretClick = () => {
  renderView.value = "create";
};

const secretCreateSubmit = () => {
  renderView.value = "list";
};

const secretCreateCancel = () => {
  renderView.value = "list";
};

const enableNewSecretButton = computed((): boolean => {
  const inEditMode = editMode.value != undefined && editMode.value;
  const notInCreateSecretView = renderView.value != "create";

  return inEditMode && notInCreateSecretView;
});
</script>
