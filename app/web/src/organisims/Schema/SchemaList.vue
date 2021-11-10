<template>
  <div
    class="flex flex-row items-center justify-between flex-grow-0 flex-shrink-0 h-12 header-background"
  >
    <div class="mt-1 ml-8 font-medium align-middle">Schema</div>
    <div class="mt-1 mr-8 align-middle">
      <ChangeSetWidget />
      <SiButton icon="plus" label="New" size="xs" @click="schemaNew()" />
    </div>
  </div>
</template>

<script setup>
import SiButton from "@/atoms/SiButton.vue";
import ChangeSetWidget from "@/organisims/ChangeSetWidget.vue";
import { useRouter } from "vue-router";

const router = useRouter();

const schemaNew = async () => {
  await router.push({ name: "schema-new" });
};
</script>

<style scoped>
.page-background {
  background-color: #1e1e1e;
}

.header-background {
  background-color: #171717;
}
</style>
