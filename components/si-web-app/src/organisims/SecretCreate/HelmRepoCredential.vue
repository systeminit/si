<template>
  <div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="pr-2 text-gray-400 align-middle w-1/2 text-right">
        <label for="username">Helm Repository Username:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          name="username"
          placeholder=""
          id="username"
          required
          v-model="username"
          @input="updateInput"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
        <label for="password">Helm Repository Password:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          name="password"
          placeholder=""
          id="password"
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
  name: "HelmRepoCredential",
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
