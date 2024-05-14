<template>
  <WorkspaceVizSchemaVariant />
</template>

<script lang="ts" setup>
import { useRouter } from "vue-router";
import { useWorkspacesStore } from "@/store/workspaces.store";
import WorkspaceVizSchemaVariant from "./WorkspaceVizSchemaVariant.vue";

const workspacesStore = useWorkspacesStore();
const workspacePk = workspacesStore.getAutoSelectedWorkspacePk();

const router = useRouter();
router.replace({
  name: "workspace-single",
  params: { workspacePk },
});
</script>
