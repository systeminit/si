<template>
  <div class="flex flex-col shadow-lg select-none login-form w-96">
    <div
      class="flex items-center justify-between pt-1 pb-1 pl-1 text-sm text-white bg-black"
    >
      <div class="px-2 login-form-header">Welcome to System Initiative!</div>
    </div>
    <div class="p-4">
      <div
        v-if="errorMessage"
        data-testid="error-message"
        class="text-white bg-red-500"
      >
        Error: {{ errorMessage }}
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label class="login-form-text" for="billingAccountName"
            >Billing Account Name</label
          >
        </div>
        <div class="w-2/3 align-middle">
          <input
            id="billingAccountName"
            v-model="form.billingAccountName"
            data-test="billingAccountName"
            class="block w-full px-2 py-1 pr-8 leading-tight shadow login-form-input focus:outline-none"
            :class="inputStyling('billingAccount')"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label class="login-form-text" for="userEmail">Email</label>
        </div>
        <div class="w-2/3 align-middle">
          <input
            id="userEmail"
            v-model="form.userEmail"
            data-test="userEmail"
            class="block w-full px-2 py-1 pr-8 leading-tight shadow login-form-input focus:outline-none"
            :class="inputStyling('email')"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
          <label class="login-form-text" for="password">Password</label>
        </div>
        <div class="w-2/3 align-middle">
          <input
            id="password"
            v-model="form.userPassword"
            data-test="password"
            class="block w-full px-2 py-1 pr-8 leading-tight shadow login-form-input focus:outline-none"
            :class="inputStyling('password')"
            type="password"
          />
        </div>
      </div>
    </div>

    <div class="flex justify-end w-full p-2">
      <div class="pr-2">
        <button
          class="inline-block py-1 button button-signup"
          aria-label="Sign Up"
          @click="goToSignUp"
        >
          [Sign Up]
        </button>
      </div>
      <div class="pr-2">
        <button
          data-test="login"
          class="inline-block py-1 button button-login"
          :class="loginButtonStyling()"
          aria-label="Sign In"
          @click="login"
        >
          [Login]
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { SessionService } from "@/service/session";

enum InputKind {
  BillingAccount = "billingAccount",
  Email = "email",
  Password = "password",
}

interface IData {
  form: {
    billingAccountName: string;
    userEmail: string;
    userPassword: string;
  };
  errorMessage: string | undefined;
}

export default defineComponent({
  name: "LoginForm",
  components: {},
  emits: ["signup", "success"],
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
      const reply = await SessionService.login(this.form);
      if (reply.error) {
        this.errorMessage = "Login error; please try again!";
      } else {
        //user$.next(reply.user);
        //billingAccount$.next(reply.billingAccount);
        this.$emit("success");
      }
    },
    inputStyling(inputKind: string): Record<string, any> {
      let classes: Record<string, any> = {};

      if (
        (inputKind == InputKind.BillingAccount &&
          this.form.billingAccountName) ||
        (inputKind == InputKind.Email && this.form.userEmail) ||
        (inputKind == InputKind.Password && this.form.userPassword)
      ) {
        classes["login-form-input-validated"] = true;
      }
      return classes;
    },
    loginButtonStyling(): Record<string, any> {
      let classes: Record<string, any> = {};

      if (
        this.form.billingAccountName &&
        this.form.userEmail &&
        this.form.userPassword
      ) {
        classes["button-login-validated"] = true;
      }
      return classes;
    },
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 2;
$button-brightness: 1.05;

.login-form {
  background-color: #151b1e;
  border: 1px solid #2c3940;
}

.login-form-header {
  font-family: "Source Code Pro";
  font-size: 0.85em;
  font-weight: 500;
}

.login-form-text {
  font-family: "Source Code Pro";
  font-size: 0.85em;
  font-weight: 400;
  color: #dcdddd;
}

.login-form-input {
  font-family: "Source Code Pro";
  font-size: 0.845em;
  font-weight: 400;
  background-color: #151b1e;
  color: #e5e5e5;
  border-bottom: 1px;
  border-style: solid;
  border-color: #b5b5b5;
}

.login-form-input-validated {
  border-color: #9ab5a4;
}

.button-signup {
  color: #ccfdff;
}

.button-login {
  color: #cccdcd;
}

.button-login-validated {
  color: #b3ffcf;
}

.button {
  font-family: "Source Code Pro";
  font-size: 0.8em;
}

.button:hover {
  filter: saturate($button-saturation) brightness($button-brightness);
}

.button:focus {
  outline: none;
}

.button:active {
  filter: saturate(2) brightness($button-brightness);
}
</style>
