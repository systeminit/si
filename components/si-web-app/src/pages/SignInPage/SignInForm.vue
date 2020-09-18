<!-- display: block -->

<template>
  <div id="signin-form">
    <form>
      <div class="flex flex-col border border-teal-800 w-80">
        <div class="flex items-center h-16 px-4 py-2 bg-teal-800">
          <p class="text-xl text-left text-white ">{{ form.title.label }}</p>
        </div>

        <div class="flex flex-col px-8 py-4">
          <div class="flex flex-row items-center h-12 group">
            <div class="pr-4">
              <UserIcon
                size="1.5x"
                class="text-teal-700 custom-class group-hover:text-teal-500"
              ></UserIcon>
            </div>
            <div
              class="flex-1 border-b border-b-2 border-teal-800 group group-hover:border-teal-500"
            >
              <input
                class="w-full px-2 py-1 mr-3 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none"
                type="text"
                :placeholder="form.account.label"
                :aria-label="form.account.id"
                data-cy="billingAccountName"
                v-model="objVariables[form.account.id]"
              />
            </div>
          </div>

          <div class="flex flex-row items-center h-12 group">
            <div class="pr-4">
              <mail-icon
                size="1.5x"
                class="text-teal-700 custom-class group-hover:text-teal-500"
              ></mail-icon>
            </div>
            <div
              class="flex-1 border-b border-b-2 border-teal-800 group-hover:border-teal-500"
            >
              <input
                class="w-full px-2 py-1 mr-3 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none"
                type="text"
                :placeholder="form.email.label"
                :aria-label="form.email.id"
                data-cy="userEmail"
                v-model="objVariables[form.email.id]"
              />
            </div>
          </div>

          <div class="flex flex-row items-center h-12 group">
            <div class="pr-4">
              <lock-icon
                size="1.5x"
                class="text-teal-700 custom-class group-hover:text-teal-500"
              ></lock-icon>
            </div>
            <div
              class="flex-1 border-b border-b-2 border-teal-800 group group-hover:border-teal-500"
            >
              <input
                class="w-full px-2 py-1 mr-3 leading-tight text-gray-400 bg-transparent border-none appearance-none focus:outline-none"
                type="password"
                :placeholder="form.password.label"
                :aria-label="form.password.id"
                data-cy="userPassword"
                v-model="objVariables[form.password.id]"
              />
            </div>
          </div>
        </div>

        <div class="flex flex-row-reverse pb-4 pr-8">
          <button
            class="px-4 py-2 text-white bg-teal-700 hover:bg-teal-600"
            data-cy="signInButton"
            @click="checkLogin()"
            type="button"
          >
            {{ form.okButton.label }}
          </button>
        </div>

        <div
          class="flex flex-col text-red-700 bg-red-100 border border-red-400"
          role="alert"
          v-if="error"
        >
          <strong class="py-2 font-bold text-center bg-red-200"
            >Sign in failed; try again?</strong
          >
          <span class="px-4 py-3 text-sm font-normal">({{ error }})</span>
        </div>
      </div>
    </form>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { UserIcon, MailIcon, LockIcon } from "vue-feather-icons";
import { registry, PropMethod } from "si-registry";
import { tracer } from "@/utils/telemetry";

const user = registry.get("user");

export default Vue.extend({
  name: "SignInForm",
  components: {
    UserIcon,
    MailIcon,
    LockIcon,
  },
  data(): Record<string, any> {
    if (user == undefined) {
      throw "Registry object is undefined! Bug!";
    }
    let me = user.methods.getEntry("loginInternal") as PropMethod;
    return {
      obj: user,
      objVariables: user.graphql.variablesObject({
        methodName: "loginInternal",
      }),
      objMethod: me.request,
      error: undefined,
      form: {
        /**
         * UX Improvement
         *
         * we should remove account from the signin screen. Most users won't be
         * part of multiple orgs. If a user (email) exists into multiple orgs,
         * we should show an optional dropdown menu from the signin menu, so the
         * user can pick which account to signin into.
         */
        title: {
          label: "Sign In",
        },
        account: {
          id: "billingAccountName",
          label: "Account Name",
        },
        email: {
          id: "email",
          label: "Email",
        },
        password: {
          id: "password",
          label: "Password",
        },
        okButton: {
          label: "SignIn",
        },
      },
    };
  },
  methods: {
    async checkLogin() {
      await this.$store.dispatch("user/login", this.objVariables);
      this.$router.push("/");
    },
  },
});
</script>
