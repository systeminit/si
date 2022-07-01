<template>
  <div class="flex flex-col items-center justify-center h-screen bg-white">
    <div class="text-center">
      <p class="text-5xl text-black">New app!</p>
    </div>

    <router-view class="flex" />

    <div class="flex m-6 border-2 p-2 rounded-lg border-black">
      <OldAppSwitch />
    </div>
  </div>
</template>

<script setup lang="ts">
import OldAppSwitch from "@/atoms/OldAppSwitch.vue";
import { onMounted } from "vue";
import { SessionService } from "@/service/session";
import { useRouter, useRoute } from "vue-router";
import { firstValueFrom } from "rxjs";

const route = useRoute();
const router = useRouter();

onMounted(async () => {
  const defaults = await firstValueFrom(SessionService.getDefaults());
  if (route.name == "new" && !defaults.error) {
    await router.push({
      name: "workspace-list",
    });
  }
});
</script>
