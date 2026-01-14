<template>
  <div
    v-if="!closed"
    :class="
      clsx(
        'flex flex-col gap-md border rounded-sm p-sm mt-xs leading-snug',
        themeClasses('bg-neutral-200 border-neutral-400', 'bg-neutral-800 border-neutral-600'),
      )
    "
  >
    <div class="flex flex-col gap-xs">
      <div class="flex flex-row justify-between">
        <span v-if="hasUsedAiAgent" class="font-medium">
          Get started with these prompts in our
          <a class="font-medium underline" href="https://github.com/systeminit/si-ai-agent" target="_blank">
            AI Agent:
          </a>
        </span>
        <span v-else class="font-medium">
          Set up the AI agent and run these prompts to see System Initiative in action:
        </span>
        <Icon
          v-if="hasUsedAiAgent"
          name="x"
          size="sm"
          class="cursor-pointer hover:scale-110 rounded-full opacity-80 hover:opacity-100 self-start"
          @click="closed = true"
        />
        <div v-else class="flex flex-row gap-sm">
          <NewButton label="Learn More" href="https://docs.systeminit.com/tutorials/getting-started" target="_blank" />
          <NewButton aria-label="Go to Onboarding" label="Get started" tone="action" @click="ctx.reopenOnboarding" />
        </div>
      </div>
    </div>
    <div class="flex desktop:flex-row flex-col desktop:gap-md gap-xs">
      <CopyableTextBlock v-for="(prompt, index) in prompts" :key="index" :text="prompt" prompt class="basis-1/3" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { Icon, NewButton, themeClasses } from "@si/vue-lib/design-system";
import { useLocalStorage } from "@vueuse/core";
import { computed } from "vue";
import { useContext } from "@/newhotness/logic_composables/context";
import CopyableTextBlock from "./CopyableTextBlock.vue";

const ctx = useContext();

const hasUsedAiAgent = computed(() => ctx.userWorkspaceFlags.value.executedAgent ?? false);

const HAS_DISMISSED_WELCOME_BANNER_KEY = "dismissed-welcome-banner";

const closed = useLocalStorage(HAS_DISMISSED_WELCOME_BANNER_KEY, false);
</script>

<script lang="ts">
export const prompts = [
  "Can you analyze my AWS infrastructure with System Initiative, discover what resources you need to, and explain what you find?  Pay special attention to what applications are running and how they are situated on the network.",
  "Create a security group that only my IP can access",
  "Discover all EC2 instances and ensure that they follow a consistent tagging mechanism to include a cost-center and and team",
];
</script>
