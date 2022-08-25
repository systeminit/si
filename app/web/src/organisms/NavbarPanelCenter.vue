<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full"
  >
    <SiBarButton
      tooltip-text="Model"
      :selected="route.name === 'workspace-compose'"
      :router-link-to="{ name: 'workspace-compose' }"
    >
      <ComposeIcon class="w-6" />
    </SiBarButton>

    <SiBarButton
      tooltip-text="Customize"
      :selected="route.name === 'workspace-lab'"
      :router-link-to="{ name: 'workspace-lab' }"
    >
      <BeakerIcon class="w-6" />
    </SiBarButton>

    <!-- Vertical bar -->
    <div class="w-0.5 h-8 self-center mx-2 bg-white"></div>

    <SiBarButton
      tooltip-text="Analyze"
      :selected="route.name === 'workspace-view'"
      :router-link-to="{ name: 'workspace-view' }"
    >
      <ViewIcon class="w-6" />
    </SiBarButton>

    <SiBarButton
      tooltip-text="Act"
      :selected="route.name === 'workspace-runtime'"
      :router-link-to="{ name: 'workspace-runtime' }"
    >
      <RunIcon class="w-6" />
    </SiBarButton>
  </div>
</template>

<script setup lang="ts">
import ComposeIcon from "@/atoms/CustomIcons/ComposeIcon.vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import ViewIcon from "@/atoms/CustomIcons/ViewIcon.vue";
import RunIcon from "@/atoms/CustomIcons/RunIcon.vue";
import BeakerIcon from "@/atoms/CustomIcons/BeakerIcon.vue";
import { useRoute } from "vue-router";

const route = useRoute();
</script>
