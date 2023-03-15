<!-- eslint-disable vue/no-v-html -->
<template>
  <div>
    <h2>Review TOS!</h2>

    <template v-if="loadTosReqStatus.isPending"> loading... </template>
    <template v-else-if="loadTosReqStatus.isError">
      Error loading TOS - {{ loadTosReqStatus.errorMessage }}
    </template>
    <template v-else-if="loadTosReqStatus.isSuccess">
      <div class="bg-red-500" v-html="authStore.tosDetails?.html" />
      <input v-model="userAgreed" type="checkbox" />

      <!-- TODO: swap in vbutton which will simplify this... -->
      <button :disabled="disableContinueButton" @click="agreeButtonHandler">
        <template v-if="agreeTosReqStatus.isPending">loading...</template>
        <template v-else>I agree!</template>
      </button>
    </template>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";
import { computed, ref, watch } from "vue";
import { useAuthStore } from "@/store/auth.store";

const authStore = useAuthStore();
const router = useRouter();

const loadTosReqStatus = authStore.getRequestStatus("LOAD_TOS_DETAILS");
const agreeTosReqStatus = authStore.getRequestStatus("AGREE_TOS");

const userAgreed = ref(false);

const disableContinueButton = computed(() => {
  if (!userAgreed.value) return true;
  if (agreeTosReqStatus.value.isPending) return true;
  return false;
});

async function loadTosDetails() {
  if (authStore.user?.needsTosUpdate === false) {
    return router.push({ name: "dashboard" });
  }
  await authStore.LOAD_TOS_DETAILS();
}

watch(() => authStore.user?.needsTosUpdate, loadTosDetails, {
  immediate: true,
});

async function agreeButtonHandler() {
  const agreeReq = await authStore.AGREE_TOS();
  if (agreeReq.result.success) {
    await router.push({ name: "dashboard" });
  }
}
</script>
