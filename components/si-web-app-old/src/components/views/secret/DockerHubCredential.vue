<template>
  <div>
    <div class="flex flex-row mx-2 my-2">
      <div class="text-white">dockerHub username:</div>

      <input
        class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
        type="text"
        placeholder="docker hub username"
        data-cy="new-secret-form-docker-hub-username"
        v-model="username"
        @input="updateInput"
      />
    </div>
    <div class="flex flex-row mx-2 my-2">
      <div class="text-white">dockerHub password:</div>

      <input
        class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
        type="password"
        placeholder="docker hub password"
        data-cy="new-secret-form-docker-hub-password"
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
  name: "DockerHubCredential",
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
