<template>
  <div>
    <div class="flex flex-row mx-2 my-2">
      <div class="text-white">helmRepo username:</div>

      <input
        data-cy="new-secret-form-secret-name"
        class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
        type="text"
        placeholder="helm repo username"
        v-model="username"
        @input="updateInput"
      />
    </div>
    <div class="flex flex-row mx-2 my-2">
      <div class="text-white">helmRepo password:</div>

      <input
        data-cy="new-secret-form-secret-name"
        class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
        type="password"
        placeholder="helm repo password"
        v-model="password"
        @input="updateInput"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

interface Data {
  username: string;
  password: string;
}

export default Vue.extend({
  name: "HelmRepoCredential",
  data(): Data {
    return {
      username: "",
      password: "",
    };
  },
  methods: {
    updateInput() {
      this.$emit("input", {
        username: this.username,
        password: this.password,
      });
    },
  },
});
</script>

<style scoped>
.input-bg-color {
  background-color: #25788a;
}
</style>

