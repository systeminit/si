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
            :disabled="activeView.value === View.Create"
            @click="setActiveView(View.Create)"
          />
        </div>
      </div>
    </template>
    <template #content>
      <div class="w-full">
        <SecretList v-if="activeView === View.List" />
        <div
          v-else-if="activeView === View.Create"
          class="flex items-center justify-center"
        >
          <div
            class="flex flex-grow px-4 py-4 mx-8 mt-8 border border-gray-700"
          >
            <SecretCreate
              @cancel="activeView === View.List"
              @submit="activeView === View.List"
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
import { ref } from "vue";

const props = defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: Boolean,
  isMaximizedContainerEnabled: Boolean,
});

enum View {
  Create,
  List,
}

const activeView = ref<View>(View.List);
const setActiveView = (view: View) => {
  activeView.value = view;
};
</script>
