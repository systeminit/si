<template>
  <div class="flex flex-col shadow-lg select-none w-96 signup-form">
    <div
      class="flex items-center justify-between pt-1 pb-1 pl-1 text-sm text-white bg-black"
    >
      <div class="signup-form-header">Sign Up</div>
    </div>
    <div class="p-4">
      <div class="text-white bg-red-500" v-if="errorMessage">
        Error: {{ errorMessage }}
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-2/3 pr-2 text-right text-gray-400 align-middle">
          <label class="signup-form-text" for="billingAccountName"
            >Organization</label
          >
        </div>
        <div class="w-2/3 align-middle">
          <input
            class="block w-full px-2 py-1 pr-8 leading-tight shadow signup-form-input focus:outline-none"
            :class="inputStyling('billingAccount')"
            id="billingAccountName"
            v-model="form.billingAccountName"
          />
        </div>
      </div>

      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-2/3 pr-2 text-right text-gray-400 align-middle">
          <label class="signup-form-text" for="userFullName">Full Name</label>
        </div>
        <div class="w-2/3 align-middle">
          <input
            class="block w-full px-2 py-1 pr-8 leading-tight shadow signup-form-input focus:outline-none"
            :class="inputStyling('name')"
            id="userName"
            v-model="form.userName"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-2/3 pr-2 text-right text-gray-400 align-middle">
          <label class="signup-form-text" for="userEmail">Email</label>
        </div>
        <div class="w-2/3 align-middle">
          <input
            class="block w-full px-2 py-1 pr-8 leading-tight shadow signup-form-input focus:outline-none"
            :class="inputStyling('email')"
            id="userEmail"
            v-model="form.userEmail"
          />
        </div>
      </div>
      <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-2/3 pr-2 text-right text-gray-400 align-middle">
          <label class="signup-form-text" for="userPasswordFirst"
            >Password</label
          >
        </div>
        <div class="w-2/3 align-middle">
          <input
            class="block w-full px-2 py-1 pr-8 leading-tight shadow signup-form-input focus:outline-none"
            :class="inputStyling('password')"
            id="userPasswordFirst"
            type="password"
            v-model="form.userPassword"
          />
        </div>
      </div>
      <!-- <div class="flex flex-row items-center object-center mx-2 my-2">
        <div class="w-2/3 pr-2 text-right text-gray-400 align-middle">
          <label for="userPasswordSecond">Password Again</label>
        </div>
        <div class="w-2/3 align-middle">
          <input
            class="block w-full px-2 py-1 pr-8 leading-tight shadow login-form-input focus:outline-none"
            :class="inputStyling('billingAccount')"
            id="userPasswordSecond"
            type="password"
          />
        </div>
      </div> -->
    </div>

    <div class="flex justify-end w-full p-2">
      <div class="pr-2">
        <button
          class="inline-block py-1 button button-cancel"
          aria-label="Cancel"
          @click="backToLogin"
        >
          [Cancel]
        </button>
      </div>
      <div class="pr-2">
        <button
          class="inline-block py-1 button button-signup"
          :class="signupButtonStyling()"
          aria-label="Sign Up"
          @click="createBillingAccount"
        >
          [Sign Up]
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { SignupDal } from "@/api/sdf/dal/signupDal";

enum InputKind {
  BillingAccount = "billingAccount",
  Name = "name",
  Email = "email",
  Password = "password",
}

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
  name: "SignupForm",
  components: {},
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
        this.$emit("success");
      }
    },
    async backToLogin() {
      this.$emit("back-to-login");
    },
    inputStyling(inputKind: string): Record<string, any> {
      let classes: Record<string, any> = {};

      if (
        (inputKind == InputKind.BillingAccount &&
          this.form.billingAccountName) ||
        (inputKind == InputKind.Name && this.form.userName) ||
        (inputKind == InputKind.Email && this.form.userEmail) ||
        (inputKind == InputKind.Password && this.form.userPassword)
      ) {
        classes["signup-form-input-validated"] = true;
      }
      return classes;
    },
    signupButtonStyling(): Record<string, any> {
      let classes: Record<string, any> = {};

      if (
        this.form.billingAccountName &&
        this.form.userName &&
        this.form.userEmail &&
        this.form.userPassword
      ) {
        classes["button-signup-validated"] = true;
      }
      return classes;
    },
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 2;
$button-brightness: 1.05;

.signup-form {
  background-color: #151b1e;
  border: 1px solid #2c3940;
}

.signup-form-header {
  font-family: "Source Code Pro";
  font-size: 0.85em;
  font-weight: 500;
}

.signup-form-text {
  font-family: "Source Code Pro";
  font-size: 0.85em;
  font-weight: 400;
  color: #dcdddd;
}

.signup-form-input {
  font-family: "Source Code Pro";
  font-size: 0.845em;
  font-weight: 400;
  background-color: #151b1e;
  color: #e5e5e5;
  border-bottom: 1px;
  border-style: solid;
  border-color: #b5b5b5;
}

.signup-form-input-validated {
  border-color: #9ab5a4;
}

.button-cancel {
  color: #ff99b8;
}

.button-signup {
  color: #cccdcd;
}

.button-signup-validated {
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
