<template>
  <div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
        <label for="tenant">Azure Tenant ID:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          name="tenantId"
          placeholder=""
          id="tenantId"
          required
          v-model="tenant"
          @input="updateInput"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
        <label for="servicePrincipalUri">Service Principal URI:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          name="servicePrincipalUri"
          placeholder=""
          id="servicePrincipalUri"
          required
          v-model="servicePrincipalUri"
          @input="updateInput"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
        <label for="password">Password:</label>
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
  servicePrincipalUri: string;
  password: string;
  tenant: string;
}

export default Vue.extend({
  name: "AzureServicePrincipal",
  components: {
    SiTextBox,
  },
  data(): IData {
    return {
      servicePrincipalUri: "",
      password: "",
      tenant: "",
    };
  },
  methods: {
    updateInput() {
      this.$emit("input", {
        servicePrincipalUri: this.servicePrincipalUri,
        password: this.password,
        tenant: this.tenant,
      } as IData);
    },
  },
});
</script>
