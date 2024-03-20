<template>
  <WorkspaceVizSchemaVariant />
</template>

<script lang="ts" setup>
import { useRouter } from "vue-router";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import WorkspaceVizSchemaVariant from "./WorkspaceVizSchemaVariant.vue";

const featureFlagsStore = useFeatureFlagsStore();
const workspacesStore = useWorkspacesStore();
const workspacePk = workspacesStore.getAutoSelectedWorkspacePk();

const router = useRouter();
if (!featureFlagsStore.FEAT_GRAPHVIZ) {
  router.replace({
    name: "workspace-single",
    params: { workspacePk },
  });
}
</script>
