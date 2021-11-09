<template>
  <div class="flex flex-col w-screen h-screen text-white bg-black vld-parent">
    <SiLoader v-model="isLoading"  />
    <DebugRoute testId="location-display-homepage" />
    <div class="flex flex-row w-full h-full overflow-hidden">
      <div
        class="flex flex-col flex-no-wrap items-center justify-between flex-shrink-0 bg-primary"
        v-show="navIsVisible"
      >
        <Nav />
      </div>
      <div class="flex flex-col w-full h-full bg-gray-900">
        <SiError :message="errorMessage" @clear="clearErrorMessage" />
        <router-view class="w-full h-full overflow-auto" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import Nav from "@/organisims/Nav.vue";
import SiError from "@/atoms/SiError.vue";
import SiLoader from "@/atoms/SiLoader.vue";
import DebugRoute from "@/atoms/DebugRoute.vue";
import { SessionService } from "@/api/sdf/service/session";
import { useRouter, useRoute } from "vue-router";

const route = useRoute();
const router = useRouter();
const errorMessage = ref("");
const isLoading = ref(true);
const navIsVisible = ref(true);

const clearErrorMessage = () => {
  errorMessage.value = "";
};

onMounted(async () => {
  const defaults = await SessionService.getDefaults();
  isLoading.value = false;
  if (route.name == "home" && !defaults.error) {
    await router.push({
      name: "workspace",
      params: {
        organizationId: defaults.organization.id,
        workspaceId: defaults.workspace.id,
      },
    });
  }
});
</script>
