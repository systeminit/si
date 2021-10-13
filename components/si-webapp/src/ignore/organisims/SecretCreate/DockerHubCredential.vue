<template>
  <div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="username">Docker Hub Username:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          size="xs"
          name="username"
          placeholder=""
          id="username"
          :isShowType="false"
          required
          v-model="username"
          @input="updateInput"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div
        class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle border-blue-100"
      >
        <label for="password">Docker Hub Password:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          size="xs"
          name="password"
          placeholder=""
          id="password"
          :isShowType="false"
          required
          v-model="password"
          type="password"
          @input="updateInput"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import SiTextBox from "@/atoms/SiTextBox.vue";

interface IData {
  username: string;
  password: string;
}

export default Vue.extend({
  name: "DockerHubCredential",
  components: {
    SiTextBox,
  },
  data(): IData {
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
      } as IData);
    },
  },
});
</script>
