<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full"
  >
    <SiNavbarButton
      tooltip-text="Compose yourself, dammit!"
      :selected="route.name === 'workspace-compose'"
      @click="onCompose"
    >
      <ComposeIcon class="w-6" />
    </SiNavbarButton>

    <SiNavbarButton
      tooltip-text="Are you a thrill beaker?"
      :selected="route.name === 'workspace-lab'"
      @click="onLab"
    >
      <BeakerIcon class="w-6" />
    </SiNavbarButton>

    <!-- Vertical bar -->
    <div class="w-0.5 h-8 self-center mx-2 bg-white"></div>

    <SiNavbarButton
      tooltip-text="Eye see you"
      :selected="route.name === 'workspace-view'"
      @click="onView"
    >
      <ViewIcon class="w-6" />
    </SiNavbarButton>

    <SiNavbarButton
      tooltip-text="Dookie, by Green Play"
      :selected="route.name === 'workspace-runtime'"
      @click="onRuntime"
    >
      <RunIcon class="w-6" />
    </SiNavbarButton>
  </div>
</template>

<script setup lang="ts">
import ComposeIcon from "@/atoms/CustomIcons/ComposeIcon.vue";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import ViewIcon from "@/atoms/CustomIcons/ViewIcon.vue";
import RunIcon from "@/atoms/CustomIcons/RunIcon.vue";
import BeakerIcon from "@/atoms/CustomIcons/BeakerIcon.vue";
import { useRoute } from "vue-router";
import router from "@/router";

const route = useRoute();

const onCompose = async () => {
  await router.push({
    name: "workspace-compose",
    path: "c",
  });
};

const onLab = async () => {
  await router.push({
    name: "workspace-lab",
    path: "l",
  });
};

const onView = async () => {
  await router.push({
    name: "workspace-view",
    path: "v",
  });
};

const onRuntime = async () => {
  await router.push({
    name: "workspace-runtime",
    path: "r",
  });
};
</script>
