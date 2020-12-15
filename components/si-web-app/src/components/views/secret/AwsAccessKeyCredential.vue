<template>
  <div>
    <div class="flex flex-row mx-2 my-2">
      <div class="text-white">
        accessKeyId:
      </div>

      <input
        data-cy="new-secret-form-access-key-id"
        class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
        type="text"
        placeholder="AWS Key Id"
        v-model="accessKeyId"
        @input="updateInput"
      />
    </div>
    <div class="flex flex-row mx-2 my-2">
      <div class="text-white">
        secretAccessKey:
      </div>

      <input
        data-cy="new-secret-form-secret-access-key"
        class="ml-4 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none input-bg-color"
        type="password"
        placeholder="secretAccessKey"
        v-model="secretAccessKey"
        @input="updateInput"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

interface Data {
  accessKeyId: string;
  secretAccessKey: string;
}

export default Vue.extend({
  name: "AwsAccessKeyCredential",
  data(): Data {
    return {
      accessKeyId: "",
      secretAccessKey: "",
    };
  },
  methods: {
    updateInput() {
      this.$emit("input", {
        accessKeyId: this.accessKeyId,
        secretKey: this.secretAccessKey,
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
