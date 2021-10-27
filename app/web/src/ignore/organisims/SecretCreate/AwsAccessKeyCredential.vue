<template>
  <div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="accessKeyId">AWS Access Key ID:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          size="xs"
          name="accessKeyId"
          placeholder=""
          id="accessKeyId"
          :isShowType="false"
          required
          v-model="accessKeyId"
          @input="updateInput"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="secretAccessKey">AWS Secret Access Key:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          size="xs"
          name="secretAccessKey"
          placeholder=""
          id="secretAccessKey"
          :isShowType="false"
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
