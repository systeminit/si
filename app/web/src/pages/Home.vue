<template>
  <div class="flex flex-col w-screen h-screen text-white bg-black vld-parent">
    <SiLoader v-model="isLoading" />
    <DebugRoute test-id="location-display-homepage" />
    <div class="flex flex-row w-full h-full overflow-hidden">
      <div
        v-show="navIsVisible"
        class="flex flex-col flex-no-wrap items-center justify-between flex-shrink-0 bg-primary"
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
import { SessionService } from "@/service/session";
import { useRouter, useRoute } from "vue-router";
import { globalErrorMessage$ } from "@/observable/global";
import { refFrom } from "vuse-rx";
import { map } from "rxjs/operators";
import { GlobalErrorService } from "@/service/global_error";
import { firstValueFrom } from "rxjs";

const route = useRoute();
const router = useRouter();
const errorMessage = refFrom<string>(
  globalErrorMessage$.pipe(
    map((response) => {
      if (response?.error) {
        return response.error.message;
      } else {
        return "";
      }
    }),
  ),
);
const isLoading = ref(true);
const navIsVisible = ref(true);

const clearErrorMessage = () => {
  GlobalErrorService.clear();
};

onMounted(async () => {
  const defaults = await firstValueFrom(SessionService.getDefaults());
  isLoading.value = false;
  if (route.name == "home" && !defaults.error) {
    await router.push({
      name: "application-list",
    });
  }
});
</script>
