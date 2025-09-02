<template>
  <div class="cover flex flex-col p-sm">
    <header>
      <h1 class="text-center">AWS SETUP</h1>
    </header>
    <section class="flex justify-evenly">
      <div>SECRETS</div>
      <div>REGION</div>
      <div>GO</div>
    </section>
    <footer class="flex justify-end">
      <NewButton label="SKIP" size="xs" @click="skip" />
    </footer>
  </div>
</template>

<script lang="ts" setup>
import { useRouter } from "vue-router";
import { NewButton } from "@si/vue-lib/design-system";

const router = useRouter();

const skip = () => {
  router.push({
    query: { skip: "true" },
  });
};
</script>

<style lang="css">
.cover {
  min-block-size: calc(100vh - 60px); /* navbar is 60 */
}

.cover > * {
  margin-block: 1rem;
}

.cover > :first-child:not(section) {
  margin-block-start: 0;
}

.cover > :last-child:not(section) {
  margin-block-end: 0;
}

.cover > section {
  margin-block: auto;
}
</style>
