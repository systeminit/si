<template>
  <div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="pr-2 text-gray-400 align-middle w-1/2 text-right">
        <label for="accessKeyId">AWS Access Key ID:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          name="accessKeyId"
          placeholder=""
          id="accessKeyId"
          required
          v-model="accessKeyId"
          @input="updateInput"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="pr-2 text-gray-400 align-middle w-1/2 text-right">
        <label for="secretAccessKey">AWS Secret Access Key:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          name="secretAccessKey"
          placeholder=""
          id="secretAccessKey"
          required
          v-model="secretAccessKey"
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
  accessKeyId: string;
  secretAccessKey: string;
}

export default Vue.extend({
  name: "AwsAccessKeyCredential",
  components: {
    SiTextBox,
  },
  data(): IData {
    return {
      accessKeyId: "",
      secretAccessKey: "",
    };
  },
  methods: {
    updateInput() {
      this.$emit("input", {
        accessKeyId: this.accessKeyId,
        secretAccessKey: this.secretAccessKey,
      } as IData);
    },
  },
});
</script>
