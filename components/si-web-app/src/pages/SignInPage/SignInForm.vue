<!-- display: block -->

<template>
  <div id="signin-form" :loading="$apollo.loading">
    <form>
      <div class="flex flex-col w-80 border border-teal-800">
        <div class="flex items-center bg-teal-800 px-4 py-2 h-16">
          <p class="text-left text-xl text-white ">{{ form.title.label }}</p>
        </div>

        <div class="flex flex-col px-8 py-4">
          <div class="flex flex-row items-center h-12 group">
            <div class="pr-4">
              <UserIcon
                size="1.5x"
                class="custom-class text-teal-700 group-hover:text-teal-500"
              ></UserIcon>
            </div>
            <div
              class="flex-1 group border-b border-b-2 border-teal-800 group-hover:border-teal-500"
            >
              <input
                class="appearance-none bg-transparent border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
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
                class="custom-class text-teal-700 group-hover:text-teal-500"
              ></mail-icon>
            </div>
            <div
              class="flex-1 border-b border-b-2 border-teal-800 group-hover:border-teal-500"
            >
              <input
                class="appearance-none bg-transparent border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
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
                class="custom-class text-teal-700 group-hover:text-teal-500"
              ></lock-icon>
            </div>
            <div
              class="flex-1 group border-b border-b-2 border-teal-800 group-hover:border-teal-500"
            >
              <input
                class="appearance-none bg-transparent border-none w-full text-gray-400 mr-3 py-1 px-2 leading-tight focus:outline-none"
                type="password"
                :placeholder="form.password.label"
                :aria-label="form.password.id"
                data-cy="userPassword"
                v-model="objVariables[form.password.id]"
              />
            </div>
          </div>
        </div>

        <div class="flex flex-row-reverse pr-8 pb-4">
          <button
            class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
            data-cy="signInButton"
            @click="checkLogin()"
            type="button"
          >
            {{ form.okButton.label }}
          </button>
        </div>

        <div
          class="flex flex-col bg-red-100 border border-red-400 text-red-700"
          role="alert"
          v-if="error"
        >
          <strong class="text-center font-bold bg-red-200 py-2"
            >Sign in failed; try again?</strong
          >
          <span class="font-normal text-sm px-4 py-3">({{ error }})</span>
        </div>
      </div>
    </form>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { UserIcon, MailIcon, LockIcon } from "vue-feather-icons";
import { billingAccountList, auth } from "@/utils/auth";
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
      let span = tracer.getCurrentSpan();
      if (span) {
        const parentSpan = span;
        span = tracer.startSpan("web.SignInForm.checkLogin", {
          parent: parentSpan,
        });
      } else {
        span = tracer.startSpan("web.SignInForm.checkLogin");
      }
      try {
        span.setAttributes({
          "web.SignInForm.email": this.objVariables.email,
          "web.SignInForm.billingAccountName": this.objVariables
            .billingAccountName,
        });
        const loginResult = await this.$apollo.query({
          query: user.graphql.query({
            methodName: "loginInternal",
            overrideName: "userLogin",
            overrideFields: "jwt, userId, billingAccountId",
          }),
          variables: this.objVariables,
        });
        let data = loginResult.data.userLogin;

        billingAccountList.addAccount({
          id: data.billingAccountId,
          name: this.billingAccountSelect,
        });
        this.billingAccounts = billingAccountList.getAccounts();

        await auth.login(data.jwt, data.userId);
      } catch (err) {
        this.error = err;
        console.log(err);
        return;
      }
      await this.$router.push("/");
    },
  },
});
</script>
