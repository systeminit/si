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
          Run these prompts and watch your AI agent bring AI-native
          infrastructure to life, effortlessly
        </span>
        <Icon
          name="x"
          size="sm"
          class="cursor-pointer hover:scale-110 rounded-full opacity-80 hover:opacity-100 self-start"
          @click="closed = true"
        />
      </div>
      <span class="text-sm">
        You’re currently in HEAD. Once you run a prompt, the AI will create a
        new change set (like a branch where you can do anything before changing
        your real infrastructure). Ready to start?
      </span>
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
    <div
      :class="
        clsx(
          'flex flex-row justify-between border rounded-sm px-sm py-xs items-center',
          themeClasses(
            'border-neutral-400 bg-neutral-300',
            'bg-neutral-800 border-neutral-600',
          ),
        )
      "
    >
      <span class="leading-snug">
        See how it works with your real data; reach out and have us help you to
        take the most out of System Initiative
      </span>
      <a
        :class="
          clsx(
            'p-xs border rounded-sm  whitespace-nowrap font-medium',
            themeClasses(
              'border-neutral-400 bg-neutral-200 hover:bg-neutral-400',
              'bg-neutral-700 border-neutral-600 hover:bg-neutral-600',
            ),
          )
        "
        href="https://www.systeminit.com/?modal=demo"
        target="_blank"
      >
        Schedule a demo
      </a>
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
  "Reach into my AWS account and pull out the default VPC. If additional VPCs are found, remove them from the model after discovery",
  "Start from the VPC and discover related Subnets, Route tables, NAT Gateways and VPC Gateway Attachments and Internet Gateways.",
  "Import all existing VPCs in my AWS account",
];

const copyText = (text: string) => {
  navigator.clipboard.writeText(text);
};
</script>
