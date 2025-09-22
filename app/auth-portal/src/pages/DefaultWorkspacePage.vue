<template><div>Redirect to default workspace</div></template>

<script lang="ts" setup>
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useAuthStore } from "@/store/auth.store";
import { API_HTTP_URL } from "@/store/api";

const authStore = useAuthStore();
const router = useRouter();
const workspacesStore = useWorkspacesStore();
const defaultWorkspace = computed(() => workspacesStore.defaultWorkspace);

onMounted(async () => {
  if (import.meta.env.SSR) return;
  if (
    !authStore.userIsLoggedIn ||
    !authStore.user ||
    !authStore.user.onboardingDetails?.reviewedProfile
  )
    return;

  if (authStore.user.needsTosUpdate) {
    return router.push({
      name: "review-legal",
    });
  } else if (!authStore.user.emailVerified) {
    // unverified users who have finished the ToS go to the profile page
    return router.push({
      name: "profile",
    });
  }

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  await workspacesStore.LOAD_WORKSPACES();
  if (defaultWorkspace.value) {
    window.location.href = `${API_HTTP_URL}/workspaces/${defaultWorkspace.value.id}/go`;
  } else {
    await router.push({
      name: "workspaces",
    });
  }
});
</script>
