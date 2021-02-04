<template>
  <div class="flex flex-col shadow-lg">
    <div data-testid="location-display" class="invisible">
      {{ $route.fullPath }}
    </div>
    <div
      class="flex items-center justify-between pl-1 pt-1 pb-1 text-sm text-white bg-black"
    >
      <div>Sign Up</div>
    </div>
    <div class="p-4 bg-blueGray-800">
      <div class="bg-red-500 text-white" v-if="errorMessage">
        Error: {{ errorMessage }}
      </div>
      <div class="flex flex-row mx-2 my-2 items-center object-center">
        <div class="pr-2 text-gray-400 align-middle w-1/2 text-right">
          <label for="billingAccountName">Billing Account Name:</label>
        </div>
        <div class="align-middle w-1/2">
          <SiTextBox
            name="billingAccountName"
            placeholder="acme, inc."
            id="billingAccountName"
            required
            v-model="form.billingAccountName"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label for="billingAccountDescription">
            Billing Account Description:
          </label>
        </div>
        <div class="w-1/2 align-middle">
          <SiTextBox
            name="billingAccountDescription"
            placeholder="defeats wasckly wabbits"
            id="billingAccountDescription"
            required
            v-model="form.billingAccountDescription"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label for="userFullName">User Full Name:</label>
        </div>
        <div class="w-1/2 align-middle">
          <SiTextBox
            name="userFullName"
            placeholder="Bobo T. Clown"
            id="userFullName"
            required
            v-model="form.userFullName"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label for="userEmail">User E-Mail:</label>
        </div>
        <div class="w-1/2 align-middle">
          <SiTextBox
            name="userEmail"
            placeholder="bobo@tclown.com"
            id="userEmail"
            required
            v-model="form.userEmail"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label for="userPasswordFirst">User Password:</label>
        </div>
        <div class="w-1/2 align-middle">
          <SiTextBox
            name="userPasswordFirst"
            placeholder="supers3cret"
            id="userPasswordFirst"
            required
            type="password"
            v-model="form.userPassword"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label for="userPasswordSecond">User Password Again:</label>
        </div>
        <div class="w-1/2 align-middle">
          <SiTextBox
            name="userPasswordSecond"
            placeholder="supers3cret"
            id="userPasswordSecond"
            required
            type="password"
          />
        </div>
      </div>
    </div>

    <div class="flex justify-end w-full p-2 bg-blueGray-800">
      <div class="pr-2">
        <SiButton
          @click.native="backToLogin"
          label="Cancel"
          kind="cancel"
          icon="cancel"
        />
      </div>
      <div>
        <SiButton
          @click.native="createBillingAccount"
          label="Sign Up"
          kind="save"
          icon="save"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import SiTextBox from "@/components/SiTextBox.vue";
import SiButton from "@/components/SiButton.vue";
import { SignupDal } from "@/api/sdf/dal/signupDal";

interface IData {
  form: {
    billingAccountName: string;
    billingAccountDescription: string;
    userName: string;
    userEmail: string;
    userPassword: string;
  };
  errorMessage: string | undefined;
}

export default Vue.extend({
  name: "SignupWad",
  components: {
    SiTextBox,
    SiButton,
  },
  data(): IData {
    return {
      form: {
        billingAccountName: "",
        billingAccountDescription: "",
        userName: "",
        userEmail: "",
        userPassword: "",
      },
      errorMessage: undefined,
    };
  },
  methods: {
    async createBillingAccount() {
      let reply = await SignupDal.createBillingAccount(this.form);
      if (reply.error) {
        this.errorMessage = reply.error.message;
      } else {
        this.$router.push({ name: "login" });
      }
    },
    async backToLogin() {
      this.$router.push({ name: "login" });
    },
  },
});
</script>
