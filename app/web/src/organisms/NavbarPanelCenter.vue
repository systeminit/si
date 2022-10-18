<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full"
  >
    <SiBarButton
      tooltip-text="Model"
      :selected="route.name === 'workspace-compose'"
      :router-link-to="{
        name: 'workspace-compose',
        params: { changeSetId: 'auto' },
      }"
    >
      <Icon name="diagram" />
    </SiBarButton>

    <SiBarButton
      tooltip-text="Customize"
      :selected="route.name === 'workspace-lab'"
      :router-link-to="{
        name: 'workspace-lab',
        params: { changeSetId: 'auto' },
      }"
    >
      <Icon name="beaker" />
    </SiBarButton>

    <!-- Vertical bar -->
    <div class="w-0.5 h-8 self-center mx-xs bg-white"></div>

    <SiBarButton
      tooltip-text="Analyze"
      :selected="route.name === 'workspace-view'"
      :router-link-to="{ name: 'workspace-view' }"
    >
      <Icon name="eye" />
    </SiBarButton>

    <SiBarButton
      tooltip-text="Fix"
      :selected="route.name === 'workspace-fix'"
      :router-link-to="{ name: 'workspace-fix' }"
    >
      <Icon name="tools" />
    </SiBarButton>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from "vue-router";
import SiBarButton from "@/molecules/SiBarButton.vue";
import Icon from "@/ui-lib/Icon.vue";

const route = useRoute();
</script>
