<template>
  <div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="username">Docker Hub Username:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          id="username"
          v-model="username"
          size="xs"
          name="username"
          placeholder=""
          :is-show-type="false"
          required
          @input="updateUsername(username)"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div
        class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle border-blue-100"
      >
        <label for="password">Docker Hub Password:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          id="password"
          v-model="password"
          size="xs"
          name="password"
          placeholder=""
          :is-show-type="false"
          required
          type="password"
          @input="updatePassword(password)"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import SiTextBox from "@/atoms/SiTextBox.vue";

const username = ref<string>("ilikemybutt");
const password = ref<string>("");

const updateUsername = (input: string) => {
  username.value = input;
};
const updatePassword = (input: string) => {
  username.value = input;
};
</script>
