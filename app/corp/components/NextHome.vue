<template>
  <div class="flex flex-col w-full bg-neutral-800">
    <div class="relative overflow-hidden">
      <NextHeader hero-words />

      <div class="relative">
        <div class="absolute inset-0 flex flex-col" aria-hidden="true">
          <div class="flex-1" />
          <div class="flex-1 w-full bg-neutral-900" />
        </div>
        <div class="max-w-7xl mx-auto px-4 sm:px-6">
          <img
            class="relative rounded-lg"
            src="~/assets/hero-screenshot.png"
            alt="System Initiative Screenshot"
          />
        </div>
      </div>
    </div>
    <div id="features" class="relative bg-neutral-900 py-16 sm:py-24 lg:py-32">
      <div
        class="mx-auto max-w-md px-4 text-center sm:max-w-3xl sm:px-6 lg:max-w-7xl lg:px-8"
      >
        <h2
          class="text-base font-semibold uppercase tracking-wider text-gray-200"
        >
          Easier, Safer, Faster
        </h2>
        <p
          class="mt-2 text-3xl font-extrabold tracking-tight text-gray-100 sm:text-4xl"
        >
          Making your team better by fixing one papercut at a time
        </p>
        <div class="mt-12">
          <div class="grid grid-cols-1 gap-8 sm:grid-cols-2 lg:grid-cols-3">
            <div v-for="feature in features" :key="feature.name" class="pt-6">
              <div class="flow-root rounded-lg bg-neutral-700 px-6 pb-8">
                <div class="-mt-6">
                  <div>
                    <span
                      class="inline-flex items-center justify-center rounded-md bg-neutral-800 p-3 shadow-lg"
                    >
                      <component
                        :is="feature.icon"
                        class="h-6 w-6 text-neutral-100"
                        aria-hidden="true"
                      />
                    </span>
                  </div>
                  <h3
                    class="mt-8 text-lg font-medium tracking-tight text-gray-100"
                  >
                    {{ feature.name }}
                  </h3>
                  <p class="mt-5 text-base text-gray-300">
                    {{ feature.description }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="bg-neutral-800">
      <div
        class="max-w-7xl mx-auto py-12 px-4 sm:px-6 lg:py-16 lg:px-8 lg:flex lg:items-center"
      >
        <div class="lg:w-0 lg:flex-1">
          <h2
            id="newsletter-headline"
            class="text-3xl font-extrabold tracking-tight text-white sm:text-4xl"
          >
            Want to know more?
          </h2>
          <p class="mt-3 max-w-3xl text-lg leading-6 text-gray-300">
            System Initiative is still in private alpha. Want to be the first to
            know more? Sign up, and we'll notify you when we're ready to open
            up.
          </p>
        </div>
        <div class="mt-8 lg:mt-0 lg:ml-8">
          <form class="sm:flex">
            <label for="email-address" class="sr-only">Email address</label>
            <input
              id="email-address"
              name="email-address"
              type="email"
              autocomplete="email"
              required=""
              class="w-full px-5 py-3 border border-transparent placeholder-gray-500 focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white focus:border-white sm:max-w-xs rounded-md"
              placeholder="Enter your email"
            />
            <div
              class="mt-3 rounded-md shadow sm:mt-0 sm:ml-3 sm:flex-shrink-0"
            >
              <button
                type="submit"
                class="w-full flex items-center justify-center px-5 py-3 border border-transparent text-base font-medium rounded-md text-white bg-indigo-500 hover:bg-indigo-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-indigo-500"
              >
                Notify me
              </button>
            </div>
          </form>
          <p class="mt-3 text-sm text-gray-300">
            We care about the protection of your data. Read our
            {{ " " }}
            <a href="#" class="text-white font-medium underline">
              Privacy Policy.
            </a>
          </p>
        </div>
      </div>
    </div>
    <NextFooter />
  </div>
</template>

<script setup>
import { defineComponent, h } from "vue";
import { Popover, PopoverButton, PopoverPanel } from "@headlessui/vue";
import { MenuIcon, XIcon } from "@heroicons/vue/outline";
import {
  CogIcon,
  ServerIcon,
  GiftIcon,
  ClipboardCheckIcon,
  CloudUploadIcon,
  LightningBoltIcon,
  AcademicCapIcon,
  UserGroupIcon,
  PresentationChartLineIcon,
} from "@heroicons/vue/outline";

const features = [
  {
    name: "1:1 Modeling",
    description:
      "Use what you already know - we model technology like the creators do. We don't abstract you from it.",
    icon: CogIcon,
  },
  {
    name: "Intelligent Components",
    description:
      "Relationships between components allow you to express configuration naturally, quickly, and safely.",
    icon: AcademicCapIcon,
  },
  {
    name: "Discovery & Sync",
    description:
      "Discover your existing resources, and syncronize changes over time.",
    icon: ServerIcon,
  },
  {
    name: "Continuous Integration",
    description:
      "Built-in qualifications lint and validate components in real time. No more waiting.",
    icon: ClipboardCheckIcon,
  },
  {
    name: "Continuous Delivery",
    description:
      "Replace complex CD pipelines with built-in workflows, built up from our intelligent components.",
    icon: CloudUploadIcon,
  },
  {
    name: "On-demand actions",
    description:
      "Just need to apply that one Kubernetes object? Just do it. We'll keep track of the state.",
    icon: LightningBoltIcon,
  },
  {
    name: "Real time source control",
    description:
      "Integrated, real time source control with full semantic diffs.",
    icon: GiftIcon,
  },
  {
    name: "Multi-Player",
    description: "DevOps is a team sport. Work together in real time.",
    icon: UserGroupIcon,
  },
  {
    name: "Visualization",
    description:
      "Zoom in. Zoom out. Enhance. See the full scope of your environment.",
    icon: PresentationChartLineIcon,
  },
];
</script>
