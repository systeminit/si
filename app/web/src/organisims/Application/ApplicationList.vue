<template>
  <div
    id="applications"
    class="flex flex-col w-full h-full select-none page-background"
  >
    <div
      id="applications-header"
      class="flex flex-row items-center justify-between flex-grow-0 flex-shrink-0 h-12 header-background"
    >
      <div class="mt-1 ml-8 font-medium align-middle">Applications</div>
      <div class="mt-1 mr-8 align-middle">
        <SiButton icon="plus" label="New" size="xs" @click="applicationNew()" />
      </div>
      <SiModal
        v-model="applicationCreateModalShow"
        name="newApplication"
        :esc-to-close="true"
      >
        <template #title>Create new application</template>
        <template #body>
          <ApplicationCreateForm
            @create="closeCreateModal"
            @cancel="closeCreateModal"
          />
        </template>
        <template #buttons>
          <div></div>
        </template>
      </SiModal>
    </div>

    <div class="flex h-full mx-4 mt-4 overflow-auto">
      <!-- <ApplicationList linkTo="applicationEditor" /> -->
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

import SiButton from "@/atoms/SiButton.vue";
import SiModal from "@/molecules/SiModal.vue";
import ApplicationCreateForm from "@/organisims/Application/ApplicationCreateForm.vue";

const applicationCreateModalShow = ref<boolean>(false);
const applicationNew = () => {
  applicationCreateModalShow.value = true;
};
const closeCreateModal = () => {
  applicationCreateModalShow.value = false;
};
</script>

<style scoped>
.page-background {
  background-color: #1e1e1e;
}

.header-background {
  background-color: #171717;
}
</style>
