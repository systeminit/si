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
      <!-- FIXME(nick): took some liberties here with the classes since old-web does not display
      this button cleanly. At the same time, this should get re-evaluated as we clean things up.
      -->
      <div class="justify-start flew flex-grow">
        <div class="min-w-max pl-2 align-middle">
          <SiButton
            icon="plus"
            label="new"
            size="xs"
            class="pl-2 align-middle"
            :disabled="!editMode || isCreateActiveView"
            @click="setActiveView('create')"
          />
        </div>
      </div>
    </template>
    <template #content>
      <div class="w-full">
        <SecretList v-if="isListActiveView" />
        <div
          v-else-if="isCreateActiveView"
          class="flex items-center justify-center"
        >
          <div
            class="flex flex-grow px-4 py-4 mx-8 mt-8 border border-gray-700"
          >
            <SecretCreate
              v-model="activeView"
              @cancel="isListActiveView"
              @submit="isListActiveView"
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

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());
const activeView = ref<string>("list");
const setActiveView = (view: string) => {
  activeView.value = view;
};

// We use these computed booleans for reactivity. If the "activeView" changes, so do these variables.
// NOTE(nick): there is likely a more elegant way of handling reactivity, but THIS WORKS DAMMIT.
const isCreateActiveView = computed((): boolean => {
  return activeView.value === "create";
});
const isListActiveView = computed((): boolean => {
  return activeView.value === "list";
});
</script>
