<template>
  <div class="flex w-screen h-screen text-gray-200 bg-black">
    <div class="flex flex-col">
      <div class="flex flex-row m-2">
        <div class="content-end justify-end align-middle w-72">
          <label for="billingAccountName">billingAccountName</label>
        </div>
        <div class="justify-start text-black">
          <input
            id="billingAccountName"
            type="text"
            v-model="billingAccountName"
          />
        </div>
      </div>
      <div class="flex flex-row m-2">
        <div class="justify-end align-middle w-72">
          <label for="billingAccountDescription"
            >billingAccountDescription</label
          >
        </div>
        <div class="justify-start text-black">
          <input
            id="billingAccountDescription"
            type="text"
            v-model="billingAccountDescription"
          />
        </div>
      </div>
      <div class="flex flex-row m-2">
        <div class="justify-end align-middle w-72">
          <label for="userName">usersFullName</label>
        </div>
        <div class="justify-start text-black">
          <input id="userName" type="text" v-model="userName" />
        </div>
      </div>
      <div class="flex flex-row m-2">
        <div class="justify-end align-middle w-72">
          <label for="userEmail">userEmailAddress</label>
        </div>
        <div class="justify-start text-black">
          <input id="userEmail" type="text" v-model="userEmail" />
        </div>
      </div>
      <div class="flex flex-row m-2">
        <div class="justify-end align-middle w-72">
          <label for="userPassword">userPassword</label>
        </div>
        <div class="justify-start text-black">
          <input type="password" id="userPassword" v-model="userPassword" />
        </div>
      </div>
      <div class="flex flex-row m-2">
        <div class="justify-end align-middle w-72">
          <label for="userPasswordAgain">userPasswordAgain</label>
        </div>
        <div class="justify-start text-black">
          <input
            type="password"
            id="userPasswordAgain"
            v-model="userPasswordAgain"
          />
        </div>
      </div>
      <Button2
        @click.native="create"
        label="create"
        icon="save"
        kind="save"
        size="xl"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { IBillingAccountCreateRequest } from "@/api/sdf/model/billingAccount";
import Button2 from "@/components/ui/Button2.vue";

interface IData extends IBillingAccountCreateRequest {
  userPasswordAgain: string;
}

export default Vue.extend({
  name: "SignUpPage",
  components: {
    Button2,
  },
  data(): IData {
    return {
      billingAccountName: "",
      billingAccountDescription: "",
      userName: "",
      userEmail: "",
      userPassword: "",
      userPasswordAgain: "",
    };
  },
  methods: {
    async create(): Promise<void> {
      const result = await this.$store.dispatch("billingAccount/create", {
        billingAccountName: this.billingAccountName,
        billingAccountDescription: this.billingAccountDescription,
        userName: this.userName,
        userEmail: this.userEmail,
        userPassword: this.userPassword,
      });
      //this.$router.push("/signin");
    },
  },
});
</script>
