<template>
  <div>
    <div class="flex flex-row mx-2 my-2">
      <div class="text-white">
        Api Key:
      </div>

      <input
        data-cy="new-secret-form-secret-name"
        class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
        type="text"
        placeholder="Api Key"
        v-model="key"
        @input="updateInput"
      />
    </div>
    <div class="flex flex-row mx-2 my-2">
      <div class="text-white">
        Api Secret:
      </div>

      <input
        data-cy="new-secret-form-secret-name"
        class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
        type="password"
        placeholder="Api Secret"
        v-model="secret"
        @input="updateInput"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

interface Data {
  key: string;
  secret: string;
}

export default Vue.extend({
  name: "ApiClient",
  data(): Data {
    return {
      key: "",
      secret: "",
    };
  },
  methods: {
    updateInput() {
      this.$emit("input", {
        key: this.key,
        secret: this.secret,
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
