<template>
  <div
    v-if="!closed"
    :class="
      clsx(
        'flex flex-col gap-md border rounded-sm p-sm mt-xs mb-md leading-snug',
        themeClasses(
          'border-neutral-400 bg-neutral-200',
          'bg-transparent border-neutral-600',
        ),
      )
    "
  >
    <div class="flex flex-col gap-xs">
      <div class="flex flex-row justify-between">
        <span class="font-medium">
          Get started with these prompts in our
          <a
            class="font-medium underline"
            href="https://github.com/systeminit/si-ai-agent"
            target="_blank"
          >
            AI Agent:
          </a>
        </span>
        <Icon
          name="x"
          size="sm"
          class="cursor-pointer hover:scale-110 rounded-full opacity-80 hover:opacity-100 self-start"
          @click="closed = true"
        />
      </div>
    </div>
    <div class="flex flex-row gap-md">
      <div
        v-for="(prompt, index) in prompts"
        :key="index"
        :class="
          clsx(
            'flex flex-row gap-sm items-center justify-between border rounded-sm p-sm cursor-pointer active:bg-neutral-700 italic select-none basis-1/3',
            themeClasses(
              'border-neutral-400 bg-neutral-300 hover:bg-neutral-400 active:bg-neutral-500',
              'bg-neutral-800 border border-neutral-600 hover:bg-neutral-600 active:bg-neutral-700',
            ),
          )
        "
        @click="copyText(prompt)"
      >
        <span>{{ prompt }}</span>
        <Icon name="copy" size="sm" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { useLocalStorage } from "@vueuse/core";

const HAS_DISMISSED_WELCOME_BANNER_KEY = "dismissed-welcome-banner";

const closed = useLocalStorage(HAS_DISMISSED_WELCOME_BANNER_KEY, false);

const prompts = [
  "Can you analyze my AWS infrastructure with System Initiative, discover what resources you need to, and explain what you find?  Pay special attention to what applications are running and how they are situated on the network.",
  "Create a security group that only my IP can access",
  "Discover all EC2 instances and ensure that they follow a consistent tagging mechanism to include a cost-center and and team",
];

const copyText = (text: string) => {
  navigator.clipboard.writeText(text);
};
</script>
