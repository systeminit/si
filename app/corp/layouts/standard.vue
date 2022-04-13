<template>
  <div class="flex flex-col w-full">
    <div class="flex flex-row">
      <SecretCode v-model="showSecretCode" />
    </div>
    <div class="flex flex-row">
      <slot />
    </div>
    <div
      v-if="secretStore.secretAgent"
      class="flex justify-center w-full bg-red-900 p-2"
    >
      <div class="font-mono">FNORD</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
const secretStore = useSecretStore();

const showSecretCode = ref(false);

const keyListener = (event: KeyboardEvent) => {
  if (event.key === "~") {
    event.preventDefault();
    showSecretCode.value = true;
  }
};

onMounted(() => {
  window.addEventListener("keydown", keyListener);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", keyListener);
});
</script>
