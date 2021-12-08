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
            @create="created($event)"
            @cancel="canceled($event)"
          />
        </template>
        <template #buttons>
          <div></div>
        </template>
      </SiModal>
    </div>

    <div class="flex flex-col px-4 mt-4 overflow-auto">
      <div
        v-for="appEntry in applicationList"
        :key="appEntry.application.id"
        class="mb-6"
      >
        <router-link
          :to="{
            name: 'application-view',
            params: { applicationId: appEntry.application.id },
          }"
        >
          <ApplicationCard
            :application="appEntry.application"
          ></ApplicationCard>
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

import SiButton from "@/atoms/SiButton.vue";
import SiModal from "@/molecules/SiModal.vue";
import ApplicationCreateForm from "@/organisims/Application/ApplicationCreateForm.vue";
import { CreateApplicationResponse } from "@/service/application/create_application";
import { ApiResponse } from "@/api/sdf";
import { refFrom } from "vuse-rx/src/index";
import { switchMap } from "rxjs/operators";
import { GlobalErrorService } from "@/service/global_error";
import { from } from "rxjs";
import { ListApplicationItem } from "@/service/application/list_application";
import { ApplicationService } from "@/service/application";
import ApplicationCard from "@/organisims/Application/ApplicationCard.vue";
import { useRouter } from "vue-router";

const applicationList = refFrom<Array<ListApplicationItem>>(
  ApplicationService.listApplications().pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        return from([response.list]);
      }
    }),
  ),
);

const applicationCreateModalShow = ref<boolean>(false);
const applicationNew = () => {
  applicationCreateModalShow.value = true;
};

const closeCreateModal = (response: ApiResponse<CreateApplicationResponse>) => {
  if (!response.error) {
    applicationCreateModalShow.value = false;
  }
};

const canceled = (response: ApiResponse<CreateApplicationResponse>) => {
  closeCreateModal(response);
};

const router = useRouter();

const created = (response: ApiResponse<CreateApplicationResponse>) => {
  closeCreateModal(response);
  if (!response.error) {
    router
      .push({
        name: "application-view",
        params: { applicationId: response.application.id },
      })
      .catch((_e) => {
        GlobalErrorService.set({
          error: {
            code: 42,
            statusCode: 500,
            message: "cannot route to application",
          },
        });
      });
  }
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
