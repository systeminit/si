<template>
  <div class="flex flex-col shadow-lg select-none login-form">
    <div
      class="flex items-center justify-between pt-1 pb-1 pl-1 text-sm text-white bg-black"
    >
      <div>Welcome to the System Initiative!</div>
    </div>
    <div class="p-4">
      <div
        data-testid="error-message"
        class="text-white bg-red-500"
        v-if="errorMessage"
      >
        Error: {{ errorMessage }}
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label for="billingAccountName">Billing Account Name:</label>
        </div>
        <div class="w-1/2 align-middle">
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
          <label for="userPassword">User Password:</label>
        </div>
        <div class="w-1/2 align-middle">
          <SiTextBox
            name="userPassword"
            placeholder="supers3cret"
            id="userPassword"
            required
            type="password"
            v-model="form.userPassword"
          />
        </div>
      </div>
    </div>

    <div class="flex justify-end w-full p-2">
      <div class="pr-2">
        <SiButton
          label="Sign Up"
          kind="standard"
          icon="signup"
          @click.native="goToSignUp"
        />
      </div>
      <div>
        <SiButton
          label="Login"
          kind="save"
          icon="login"
          @click.native="login"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiButton from "@/atoms/SiButton.vue";
import { SessionDal } from "@/api/sdf/dal/sessionDal";
import { user$, billingAccount$ } from "@/observables";

interface IData {
  form: {
    billingAccountName: string;
    userEmail: string;
    userPassword: string;
  };
  errorMessage: string | undefined;
}

export default Vue.extend({
  name: "LoginForm",
  components: {
    SiTextBox,
    SiButton,
  },
  data(): IData {
    return {
      form: {
        billingAccountName: "",
        userEmail: "",
        userPassword: "",
      },
      errorMessage: undefined,
    };
  },
  methods: {
    goToSignUp() {
      this.$emit("signup");
    },
    async login() {
      const reply = await SessionDal.login({ ...this.form });
      if (reply.error) {
        this.errorMessage = "Login error; please try again!";
      } else {
        user$.next(reply.user);
        billingAccount$.next(reply.billingAccount);
        this.$emit("success");
      }
    },
  },
});
</script>

<style scoped>
.login-form {
  background-color: #151b1e;
  border: 1px solid #2c3940;
}
</style>
