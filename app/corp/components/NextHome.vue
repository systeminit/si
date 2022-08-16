<template>
  <div class="flex flex-col w-full bg-neutral-700">
    <div class="relative overflow-hidden">
      <NextHeader hero-words />

      <div class="relative">
        <div class="absolute inset-0 flex flex-col" aria-hidden="true">
          <div class="flex-1" />
          <div class="flex-1 w-full bg-neutral-700" />
        </div>
        <div class="max-w-7xl mx-auto px-4 sm:px-6 pb-24">
          <img
            class="relative rounded-lg"
            src="~/assets/hero-screenshot.png"
            alt="System Initiative Screenshot"
          />
        </div>
      </div>
    </div>

    <div class="py-16 bg-neutral-800 overflow-hidden lg:py-24">
      <div class="relative max-w-xl mx-auto px-4 sm:px-6 lg:px-8 lg:max-w-7xl">
        <div class="relative">
          <h2
            class="text-center text-3xl leading-8 font-bold tracking-tight text-gray-100 sm:text-4xl sm:tracking-tight"
          >
            Model, Act, Analyze, Customize
          </h2>
          <p class="mt-4 max-w-3xl mx-auto text-center text-xl text-gray-300">
            System Initiative automates the entire DevOps lifecycle. Model your
            infrastructure and applications; take action to manage their state;
            analyze them over time; customize the System into the perfect
            platform for your team.
          </p>
        </div>

        <div
          class="relative mt-12 lg:mt-24 lg:grid lg:grid-cols-2 lg:gap-8 lg:items-center"
        >
          <div class="relative">
            <h3
              class="text-2xl font-bold text-gray-100 tracking-tight sm:text-3xl sm:tracking-tight"
            >
              Model
            </h3>
            <p class="mt-3 text-lg text-gray-300">
              Fully model your infrastructure and applications like you would
              with Terraform, Pulumi, or CloudFormation. Only way faster.
            </p>

            <dl class="mt-10 space-y-10">
              <div
                v-for="item in modelFeatures"
                :key="item.name"
                class="relative"
              >
                <dt>
                  <div
                    class="absolute flex items-center justify-center h-12 w-12 rounded-md bg-neutral-900 text-white"
                  >
                    <component
                      :is="item.icon"
                      class="h-6 w-6"
                      aria-hidden="true"
                    />
                  </div>
                  <p class="ml-16 text-lg leading-6 font-medium text-gray-100">
                    {{ item.name }}
                  </p>
                </dt>
                <dd class="mt-2 ml-16 text-base text-gray-300">
                  {{ item.description }}
                </dd>
              </div>
            </dl>
          </div>

          <div class="mt-10 -mx-4 relative lg:mt-0" aria-hidden="true">
            <img
              class="relative mx-auto rounded-md"
              width="490"
              src="~/assets/hero-model.png"
              alt=""
            />
          </div>
        </div>

        <div class="relative mt-12 sm:mt-16 lg:mt-24">
          <div
            class="lg:grid lg:grid-flow-row-dense lg:grid-cols-2 lg:gap-8 lg:items-center"
          >
            <div class="lg:col-start-2">
              <h3
                class="text-2xl font-bold text-gray-100 tracking-tight sm:text-3xl sm:tracking-tight"
              >
                Act
              </h3>
              <p class="mt-3 text-lg text-gray-300">
                Take action to manage the state of your real world resources.
                Like Ansible and GitHub Actions had a delightful baby.
              </p>

              <dl class="mt-10 space-y-10">
                <div
                  v-for="item in actFeatures"
                  :key="item.id"
                  class="relative"
                >
                  <dt>
                    <div
                      class="absolute flex items-center justify-center h-12 w-12 rounded-md bg-neutral-900 text-white"
                    >
                      <component
                        :is="item.icon"
                        class="h-6 w-6"
                        aria-hidden="true"
                      />
                    </div>
                    <p
                      class="ml-16 text-lg leading-6 font-medium text-gray-100"
                    >
                      {{ item.name }}
                    </p>
                  </dt>
                  <dd class="mt-2 ml-16 text-base text-gray-300">
                    {{ item.description }}
                  </dd>
                </div>
              </dl>
            </div>

            <div class="mt-10 -mx-4 relative lg:mt-0 lg:col-start-1">
              <svg
                class="absolute left-1/2 transform -translate-x-1/2 translate-y-16 lg:hidden"
                width="784"
                height="404"
                fill="none"
                viewBox="0 0 784 404"
                aria-hidden="true"
              >
                <defs>
                  <pattern
                    id="e80155a9-dfde-425a-b5ea-1f6fadd20131"
                    x="0"
                    y="0"
                    width="20"
                    height="20"
                    patternUnits="userSpaceOnUse"
                  >
                    <rect
                      x="0"
                      y="0"
                      width="4"
                      height="4"
                      class="text-gray-200"
                      fill="currentColor"
                    />
                  </pattern>
                </defs>
                <rect
                  width="784"
                  height="404"
                  fill="url(#e80155a9-dfde-425a-b5ea-1f6fadd20131)"
                />
              </svg>
              <img
                class="relative mx-auto rounded-md"
                width="490"
                src="~/assets/hero-act.png"
                alt=""
              />
            </div>
          </div>
        </div>

        <div
          class="relative mt-12 lg:mt-24 lg:grid lg:grid-cols-2 lg:gap-8 lg:items-center"
        >
          <div class="relative">
            <h3
              class="text-2xl font-bold text-gray-100 tracking-tight sm:text-3xl sm:tracking-tight"
            >
              Analyze
            </h3>
            <p class="mt-3 text-lg text-gray-300">
              Analyze your real infrastructure and applications. Validate
              compliance, view cost breakdowns, and troubleshoot issues.
            </p>

            <dl class="mt-10 space-y-10">
              <div
                v-for="item in analyzeFeatures"
                :key="item.name"
                class="relative"
              >
                <dt>
                  <div
                    class="absolute flex items-center justify-center h-12 w-12 rounded-md bg-neutral-900 text-white"
                  >
                    <component
                      :is="item.icon"
                      class="h-6 w-6"
                      aria-hidden="true"
                    />
                  </div>
                  <p class="ml-16 text-lg leading-6 font-medium text-gray-100">
                    {{ item.name }}
                  </p>
                </dt>
                <dd class="mt-2 ml-16 text-base text-gray-300">
                  {{ item.description }}
                </dd>
              </div>
            </dl>
          </div>

          <div class="mt-10 -mx-4 relative lg:mt-0" aria-hidden="true">
            <img
              class="relative mx-auto rounded-md"
              width="490"
              src="~/assets/hero-analyze.png"
              alt=""
            />
          </div>
        </div>

        <div class="relative mt-12 sm:mt-16 lg:mt-24">
          <div
            class="lg:grid lg:grid-flow-row-dense lg:grid-cols-2 lg:gap-8 lg:items-center"
          >
            <div class="lg:col-start-2">
              <h3
                class="text-2xl font-bold text-gray-100 tracking-tight sm:text-3xl sm:tracking-tight"
              >
                Customize
              </h3>
              <p class="mt-3 text-lg text-gray-300">
                Create your own models, change behavior, author custom workflows
                - the sky is the limit. System Initiative is designed to become
                the perfect platform for your team.
              </p>

              <dl class="mt-10 space-y-10">
                <div
                  v-for="item in customizeFeatures"
                  :key="item.id"
                  class="relative"
                >
                  <dt>
                    <div
                      class="absolute flex items-center justify-center h-12 w-12 rounded-md bg-neutral-900 text-white"
                    >
                      <component
                        :is="item.icon"
                        class="h-6 w-6"
                        aria-hidden="true"
                      />
                    </div>
                    <p
                      class="ml-16 text-lg leading-6 font-medium text-gray-100"
                    >
                      {{ item.name }}
                    </p>
                  </dt>
                  <dd class="mt-2 ml-16 text-base text-gray-300">
                    {{ item.description }}
                  </dd>
                </div>
              </dl>
            </div>

            <div class="mt-10 -mx-4 relative lg:mt-0 lg:col-start-1">
              <svg
                class="absolute left-1/2 transform -translate-x-1/2 translate-y-16 lg:hidden"
                width="784"
                height="404"
                fill="none"
                viewBox="0 0 784 404"
                aria-hidden="true"
              >
                <defs>
                  <pattern
                    id="e80155a9-dfde-425a-b5ea-1f6fadd20131"
                    x="0"
                    y="0"
                    width="20"
                    height="20"
                    patternUnits="userSpaceOnUse"
                  >
                    <rect
                      x="0"
                      y="0"
                      width="4"
                      height="4"
                      class="text-gray-200"
                      fill="currentColor"
                    />
                  </pattern>
                </defs>
                <rect
                  width="784"
                  height="404"
                  fill="url(#e80155a9-dfde-425a-b5ea-1f6fadd20131)"
                />
              </svg>
              <img
                class="relative mx-auto rounded-md"
                width="490"
                src="~/assets/hero-customize.png"
                alt=""
              />
            </div>
          </div>
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
  SwitchHorizontalIcon,
  ShieldCheckIcon,
  TerminalIcon,
  GlobeIcon,
  EyeIcon,
  ShieldExclamationIcon,
  CurrencyDollarIcon,
  CodeIcon,
  PencilIcon,
  DocumentDuplicateIcon,
} from "@heroicons/vue/outline";

const modelFeatures = [
  {
    name: "Intelligent",
    description:
      "Our intelligent, relationship aware models infer valid configuration automatically.",
    icon: AcademicCapIcon,
  },
  {
    name: "Safe",
    description:
      "Real-time qualifications tell you if your model will work without impacting real environements.",
    icon: ShieldCheckIcon,
  },
  {
    name: "Bi-Directional",
    description:
      "Start by importing what you already have; we will synchronize changes automatically.",
    icon: SwitchHorizontalIcon,
  },
];

const actFeatures = [
  {
    name: "Direct",
    description:
      "Run commands directly on models to manage the individual resources they represent.",
    icon: TerminalIcon,
  },
  {
    name: "Dynamic",
    description:
      "Script dynamic deployment and management workflows; if you can do it manually, you can script it with a workflow.",
    icon: GlobeIcon,
  },
  {
    name: "Reactive",
    description:
      "Trigger workflows and commands automatically when your resources change.",
    icon: LightningBoltIcon,
  },
];

const analyzeFeatures = [
  {
    name: "Visualize",
    description: "Visualize your resources and their relationships.",
    icon: PresentationChartLineIcon,
  },
  {
    name: "Troubleshoot",
    description:
      "Spot where your infrastructure and applications aren't working correctly, and take action immediately.",
    icon: ShieldExclamationIcon,
  },
  {
    name: "Comply",
    description:
      "Use itemized cost breakdowns to stay on budget; check for compliance to regulatory requirements.",
    icon: CurrencyDollarIcon,
  },
];

const customizeFeatures = [
  {
    name: "Program",
    description:
      "Everything in System Initiative is a Typescript function you can author yourself.",
    icon: CodeIcon,
  },
  {
    name: "Control",
    description:
      "If you can see it, you can change it. We never sacrifice power.",
    icon: PencilIcon,
  },
  {
    name: "Specific",
    description:
      "Create the semantics that work for your organization on top of standardized models.",
    icon: DocumentDuplicateIcon,
  },
];

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
