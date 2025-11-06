<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts">
export enum OnboardingStep {
  PICK_PROVIDER,
  INITIALIZE,
  SETUP_AI,
}

// Toggle this to true to force onboarding to show, to go through the onboarding
// without creating anything, and without needing to pass the AI agent checks.
// Debug mode also ALWAYS shows button disabled tooltips so they can be seen
// MAKE SURE YOU SET IT BACK TO FALSE WHEN YOU ARE DONE!
// MAKE SURE YOU TEST THE FUNCTIONALITY OF THE FLOW WITH DEBUG MODE TURNED OFF!
export const DEBUG_MODE = false;
export const DEBUG_ONBOARDING_SHOW_DISABLED = false;
export const DEBUG_ONBOARDING_START = OnboardingStep.PICK_PROVIDER;
export const DEBUG_PROVIDER_CHOICE = undefined;
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<template>
  <div
    data-testid="lobby"
    :class="
      clsx(
        'absolute w-screen h-screen z-[1000]',
        themeClasses('bg-white text-black', 'bg-neutral-900 text-white,'),
      )
    "
  >
    <!-- Onboarding page main body -->
    <div
      class="flex flex-col items-center w-full h-full min-w-[900px] relative"
    >
      <!-- Gradient circles-->
      <div
        class="absolute w-[50vw] h-[50vw] rounded-full z-0 -bottom-[25vw] right-2xl bg-onboardingcircle1"
      />
      <div
        class="absolute w-[50vw] h-[50vw] rounded-full z-0 -bottom-[10vw] -right-[25vw] bg-onboardingcircle2"
      />

      <!-- Everything else -->
      <div
        class="flex flex-col items-center w-full h-full min-w-[900px] relative scrollable"
      >
        <!--  Header  -->
        <div
          class="flex flex-row flex-none items-center justify-between w-full px-sm py-xs z-10"
        >
          <SiLogo class="block h-md w-md flex-none" />
          <div
            :class="
              clsx(
                'font-normal text-sm',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              )
            "
          >
            Need help? Have questions?
            <NewButton
              aria-label="schedule-meeting-header"
              :class="
                clsx(
                  'hover:underline',
                  themeClasses(
                    'text-neutral-700 hover:text-black',
                    'text-neutral-300 hover:text-white',
                  ),
                )
              "
              tone="nostyle"
              :href="scheduleWithUsLink"
              target="_blank"
              label="Schedule a meeting"
              @mousedown="onboardingTracking('schedule_meeting_header')"
            />
            with us.
          </div>
        </div>

        <!--  Form + Text in the middle  -->
        <div
          class="flex flex-row items-center grow w-full max-w-[1600px] px-lg gap-lg z-10"
        >
          <div
            v-if="currentStep === OnboardingStep.PICK_PROVIDER"
            class="flex-1 flex flex-col items-center min-w-0 gap-8"
          >
            <div
              :class="
                clsx(
                  'text-lg',
                  themeClasses('text-neutral-800', 'text-neutral-200'),
                )
              "
            >
              Select a provider to start
            </div>
            <div class="flex flex-row items-center gap-lg">
              <OnboardingProviderTile provider="AWS" @select="pickProvider" />
              <OnboardingProviderTile
                provider="Azure"
                beta
                @select="pickProvider"
              />
            </div>
          </div>
          <template v-else>
            <div class="flex-1 flex flex-col basis-1/2 min-w-0">
              <!-- Step 2: Provider setup -->
              <template v-if="currentStep === OnboardingStep.INITIALIZE">
                <!-- AWS -->
                <OnboardingStepBlock v-if="providerChoice === 'AWS'">
                  <template #header>
                    <div class="flex flex-row items-center gap-sm">
                      <Icon name="logo-aws" />
                      <div
                        :class="
                          clsx(
                            'flex-grow',
                            initializeRequestSentAndSuccessful &&
                              'text-success-200',
                          )
                        "
                      >
                        Enter an AWS Credential
                      </div>
                      <div>1/2</div>
                    </div>
                  </template>
                  <template #body>
                    <!-- Credential -->
                    <div
                      :class="
                        clsx(
                          'flex flex-col border p-sm gap-sm text-sm',
                          themeClasses(
                            'border-neutral-400',
                            'border-neutral-600',
                          ),
                        )
                      "
                    >
                      <div class="flex flex-row justify-between items-center">
                        <label for="aws-credential-name" class="basis-0 grow">
                          Name your credential
                          <RequiredAsterisk />
                        </label>
                        <input
                          id="aws-credential-name"
                          v-model="credentialName"
                          data-lpignore="true"
                          data-1p-ignore
                          data-bwignore
                          data-form-type="other"
                          :class="
                            clsx(
                              'h-lg p-xs text-sm border font-mono cursor-text basis-0 grow',
                              'focus:outline-none focus:ring-0 focus:z-20',
                              themeClasses(
                                'text-black bg-white border-neutral-400 focus:border-action-500',
                                'text-white bg-black border-neutral-600 focus:border-action-300',
                              ),
                            )
                          "
                          @focus="
                            onboardingTracking('focused_credential_name_input')
                          "
                        />
                      </div>
                      <!-- Secret Values -->
                      <ErrorMessage
                        :class="
                          clsx(
                            'rounded-sm p-xs',
                            themeClasses('bg-action-200', 'bg-action-900'),
                          )
                        "
                        tone="action"
                        variant="block"
                        noIcon
                      >
                        <div
                          class="flex flex-row items-center justify-between text-sm"
                        >
                          <div>
                            Paste the full Bash environment block into the first
                            field — we'll auto-fill the rest.
                          </div>
                          <Icon
                            v-tooltip="
                              someFieldsVisible
                                ? 'Hide All Values'
                                : 'Show All Values'
                            "
                            :name="someFieldsVisible ? 'hide' : 'eye'"
                            size="xs"
                            class="cursor-pointer z-20"
                            @click="toggleAll"
                          />
                        </div>
                      </ErrorMessage>

                      <div class="flex flex-col">
                        <div
                          v-for="(field, title) in secretFormFieldsAWS"
                          :key="title"
                          :class="'flex flex-row justify-between items-center text-sm mb-[-1px]'"
                        >
                          <label
                            class="basis-0 grow flex flex-row items-center gap-2xs"
                          >
                            {{ title }}
                            <RequiredAsterisk v-if="field.required" />
                          </label>
                          <div class="flex flex-row basis-0 grow relative">
                            <input
                              v-model="field.ref"
                              :type="field.type"
                              :class="
                                clsx(
                                  'h-lg p-xs pr-7 text-sm border font-mono cursor-text grow',
                                  'focus:outline-none focus:ring-0 focus:z-20',
                                  themeClasses(
                                    'text-black bg-white border-neutral-400 focus:border-action-500',
                                    'text-white bg-black border-neutral-600 focus:border-action-300',
                                  ),
                                )
                              "
                              :placeholder="
                                field.type === 'password'
                                  ? '*****'
                                  : 'Value will be visible'
                              "
                              data-lpignore="true"
                              data-1p-ignore
                              data-bwignore
                              data-form-type="other"
                              @paste="(ev: ClipboardEvent) => tryMatchOnPaste(ev)"
                              @focus="
                                onboardingTracking(
                                  `focused_secret_field_${title
                                    .toLowerCase()
                                    .replace(/ /g, '_')}`,
                                )
                              "
                            />
                            <Icon
                              v-tooltip="
                                field.type === 'password'
                                  ? 'Show Value'
                                  : 'Hide Value'
                              "
                              :name="field.type === 'password' ? 'eye' : 'hide'"
                              size="xs"
                              class="absolute right-xs top-[10px] cursor-pointer z-20"
                              @click="toggleVisibility(field)"
                            />
                          </div>
                        </div>
                      </div>
                    </div>
                    <!-- Region -->
                    <div
                      :class="
                        clsx(
                          'border flex flex-col p-sm gap-sm text-sm',
                          themeClasses(
                            'border-neutral-400',
                            'border-neutral-600',
                          ),
                        )
                      "
                    >
                      <div>
                        Pick a region
                        <RequiredAsterisk />
                      </div>
                      <div
                        class="flex desktop:flex-row flex-col desktop:items-center items-stretch gap-xs"
                      >
                        <div
                          v-for="region in pickerRegions"
                          :key="region.value"
                          :class="
                            clsx(
                              'flex flex-row grow items-center gap-xs',
                              'border rounded-sm cursor-pointer',
                              'desktop:p-xs p-2xs',
                              themeClasses(
                                'hover:border-neutral-700',
                                'hover:border-neutral-300',
                              ),
                              region.value === awsRegion
                                ? themeClasses(
                                    'border-neutral-700',
                                    'border-neutral-300',
                                  )
                                : themeClasses(
                                    'border-neutral-300',
                                    'border-neutral-600',
                                  ),
                            )
                          "
                          @click="selectRegion(region.value)"
                        >
                          <Icon
                            :name="
                              region.value === awsRegion
                                ? 'check-circle'
                                : 'circle-empty'
                            "
                          />
                          <div
                            class="flex flex-col justify-center align-middle"
                          >
                            <span class="text-sm">{{ region.title }}</span>
                            <span
                              :class="
                                clsx(
                                  'text-sm',
                                  themeClasses(
                                    'text-neutral-600',
                                    'text-neutral-400',
                                  ),
                                )
                              "
                            >
                              {{ region.value }}
                            </span>
                          </div>
                        </div>
                      </div>
                      <div class="flex flex-row items-center">
                        <label for="aws-region" class="basis-0 grow">
                          Or select any region
                        </label>
                        <select
                          id="aws-region"
                          v-model="awsRegion"
                          :class="
                            clsx(
                              'h-lg basis-0 grow p-xs text-sm border font-mono cursor-pointer',
                              'focus:outline-none focus:ring-0 focus:z-20',
                              themeClasses(
                                'text-black bg-white border-neutral-400 focus:border-action-500',
                                'text-white bg-black border-neutral-600 focus:border-action-300',
                              ),
                            )
                          "
                        >
                          <option
                            v-for="region in awsRegions"
                            :key="region.value"
                            :label="`${region.title} - ${region.value}`"
                          >
                            {{ region.value }}
                          </option>
                        </select>
                      </div>
                    </div>
                  </template>
                  <template #footerRight>
                    <NewButton
                      label="Previous"
                      tone="neutral"
                      @click="decrementOnboardingStep"
                    />
                    <NewButton
                      :label="initializeApiError ? 'Retry' : 'Next'"
                      :tooltip="
                        !formHasRequiredValues || DEBUG_MODE
                          ? 'You must enter your credential to continue'
                          : undefined
                      "
                      tone="action"
                      :disabled="
                        !formHasRequiredValues || DEBUG_ONBOARDING_SHOW_DISABLED
                      "
                      :loading="submitOnboardingInProgress"
                      loadingText="Saving"
                      @click="submitOnboardRequest"
                    />
                  </template>
                </OnboardingStepBlock>
                <!-- Azure -->
                <OnboardingStepBlock v-else-if="providerChoice === 'Azure'">
                  <template #header>
                    <div class="flex flex-row items-center gap-sm">
                      <Icon name="logo-azure" />
                      <div
                        :class="
                          clsx(
                            'flex-grow',
                            initializeRequestSentAndSuccessful &&
                              'text-success-200',
                          )
                        "
                      >
                        Enter an Azure Credential
                      </div>
                      <div>1/2</div>
                    </div>
                  </template>
                  <template #body>
                    <!-- Credential -->
                    <div
                      :class="
                        clsx(
                          'flex flex-col border p-sm gap-sm text-sm',
                          themeClasses(
                            'border-neutral-400',
                            'border-neutral-600',
                          ),
                        )
                      "
                    >
                      <div class="flex flex-row justify-between items-center">
                        <label for="azure-credential-name" class="basis-0 grow">
                          Name your credential
                          <RequiredAsterisk />
                        </label>
                        <input
                          id="azure-credential-name"
                          v-model="credentialName"
                          data-lpignore="true"
                          data-1p-ignore
                          data-bwignore
                          data-form-type="other"
                          :class="
                            clsx(
                              'h-lg p-xs text-sm border font-mono cursor-text basis-0 grow',
                              'focus:outline-none focus:ring-0 focus:z-20',
                              themeClasses(
                                'text-black bg-white border-neutral-400 focus:border-action-500',
                                'text-white bg-black border-neutral-600 focus:border-action-300',
                              ),
                            )
                          "
                          @focus="
                            onboardingTracking('focused_credential_name_input')
                          "
                        />
                      </div>
                      <!-- Secret Values -->
                      <ErrorMessage
                        :class="
                          clsx(
                            'rounded-sm p-xs',
                            themeClasses('bg-action-200', 'bg-action-900'),
                          )
                        "
                        tone="action"
                        variant="block"
                        noIcon
                      >
                        <div
                          class="flex flex-row items-center justify-between text-sm"
                        >
                          <div>
                            Please enter your Azure credential information
                            below.
                          </div>
                          <Icon
                            v-tooltip="
                              someFieldsVisible
                                ? 'Hide All Values'
                                : 'Show All Values'
                            "
                            :name="someFieldsVisible ? 'hide' : 'eye'"
                            size="xs"
                            class="cursor-pointer z-20"
                            @click="toggleAll"
                          />
                        </div>
                      </ErrorMessage>

                      <div class="flex flex-col">
                        <div
                          v-for="(field, title) in secretFormFieldsAzure"
                          :key="title"
                          :class="'flex flex-row justify-between items-center text-sm mb-[-1px]'"
                        >
                          <label
                            class="basis-0 grow flex flex-row items-center gap-2xs"
                          >
                            {{ title }}
                            <RequiredAsterisk v-if="field.required" />
                          </label>
                          <div class="flex flex-row basis-0 grow relative">
                            <input
                              v-model="field.ref"
                              :type="field.type"
                              :class="
                                clsx(
                                  'h-lg p-xs pr-7 text-sm border font-mono cursor-text grow',
                                  'focus:outline-none focus:ring-0 focus:z-20',
                                  themeClasses(
                                    'text-black bg-white border-neutral-400 focus:border-action-500',
                                    'text-white bg-black border-neutral-600 focus:border-action-300',
                                  ),
                                )
                              "
                              :placeholder="
                                field.type === 'password'
                                  ? '*****'
                                  : 'Value will be visible'
                              "
                              data-lpignore="true"
                              data-1p-ignore
                              data-bwignore
                              data-form-type="other"
                              @paste="(ev: ClipboardEvent) => tryMatchOnPaste(ev)"
                              @focus="
                                onboardingTracking(
                                  `focused_secret_field_${title
                                    .toLowerCase()
                                    .replace(/ /g, '_')}`,
                                )
                              "
                            />
                            <Icon
                              v-tooltip="
                                field.type === 'password'
                                  ? 'Show Value'
                                  : 'Hide Value'
                              "
                              :name="field.type === 'password' ? 'eye' : 'hide'"
                              size="xs"
                              class="absolute right-xs top-[10px] cursor-pointer z-20"
                              @click="toggleVisibility(field)"
                            />
                          </div>
                        </div>
                        <div
                          class="flex flex-row justify-between items-center text-sm mb-[-1px]"
                        >
                          <label
                            for="azure-subscription-id"
                            class="basis-0 grow flex flex-row items-center gap-2xs"
                          >
                            Subscription ID
                            <RequiredAsterisk />
                          </label>
                          <div class="flex flex-row basis-0 grow relative">
                            <input
                              id="azure-subscription-id"
                              v-model="azureSubscriptionId"
                              :type="azureSubscriptionIdFieldType"
                              :class="
                                clsx(
                                  'h-lg p-xs pr-7 text-sm border font-mono cursor-text grow',
                                  'focus:outline-none focus:ring-0 focus:z-20',
                                  themeClasses(
                                    'text-black bg-white border-neutral-400 focus:border-action-500',
                                    'text-white bg-black border-neutral-600 focus:border-action-300',
                                  ),
                                )
                              "
                              :placeholder="
                                azureSubscriptionIdFieldType === 'password'
                                  ? '*****'
                                  : 'Value will be visible'
                              "
                              data-lpignore="true"
                              data-1p-ignore
                              data-bwignore
                              data-form-type="other"
                              @focus="
                                onboardingTracking(
                                  'focused_subscription_id_input',
                                )
                              "
                            />
                            <Icon
                              v-tooltip="
                                azureSubscriptionIdFieldType === 'password'
                                  ? 'Show Value'
                                  : 'Hide Value'
                              "
                              :name="
                                azureSubscriptionIdFieldType === 'password'
                                  ? 'eye'
                                  : 'hide'
                              "
                              size="xs"
                              class="absolute right-xs top-[10px] cursor-pointer z-20"
                              @click="toggleSubscriptionIdVisibility"
                            />
                          </div>
                        </div>
                      </div>
                    </div>
                    <!-- Location -->
                    <div
                      :class="
                        clsx(
                          'border flex flex-col p-sm gap-sm text-sm',
                          themeClasses(
                            'border-neutral-400',
                            'border-neutral-600',
                          ),
                        )
                      "
                    >
                      <div>
                        Pick a location
                        <RequiredAsterisk />
                      </div>
                      <div
                        class="flex desktop:flex-row flex-col desktop:items-center items-stretch gap-xs"
                      >
                        <div
                          v-for="location in pickerLocations"
                          :key="location.value"
                          :class="
                            clsx(
                              'flex flex-row grow items-center gap-xs',
                              'border rounded-sm cursor-pointer',
                              'desktop:p-xs p-2xs',
                              themeClasses(
                                'hover:border-neutral-700',
                                'hover:border-neutral-300',
                              ),
                              location.value === azureLocation
                                ? themeClasses(
                                    'border-neutral-700',
                                    'border-neutral-300',
                                  )
                                : themeClasses(
                                    'border-neutral-300',
                                    'border-neutral-600',
                                  ),
                            )
                          "
                          @click="selectLocation(location.value)"
                        >
                          <Icon
                            :name="
                              location.value === azureLocation
                                ? 'check-circle'
                                : 'circle-empty'
                            "
                          />
                          <div
                            class="flex flex-col justify-center align-middle"
                          >
                            <span class="text-sm">{{ location.title }}</span>
                            <span
                              :class="
                                clsx(
                                  'text-sm',
                                  themeClasses(
                                    'text-neutral-600',
                                    'text-neutral-400',
                                  ),
                                )
                              "
                            >
                              {{ location.value }}
                            </span>
                          </div>
                        </div>
                      </div>
                      <div class="flex flex-row items-center">
                        <label for="azure-location" class="basis-0 grow">
                          Or select any location
                        </label>
                        <select
                          id="azure-location"
                          v-model="azureLocation"
                          :class="
                            clsx(
                              'h-lg basis-0 grow p-xs text-sm border font-mono cursor-pointer',
                              'focus:outline-none focus:ring-0 focus:z-20',
                              themeClasses(
                                'text-black bg-white border-neutral-400 focus:border-action-500',
                                'text-white bg-black border-neutral-600 focus:border-action-300',
                              ),
                            )
                          "
                        >
                          <option
                            v-for="location in azureLocations"
                            :key="location.value"
                            :label="`${location.title} - ${location.value}`"
                          >
                            {{ location.value }}
                          </option>
                        </select>
                      </div>
                    </div>
                  </template>
                  <template #footerRight>
                    <NewButton
                      label="Previous"
                      tone="neutral"
                      @click="decrementOnboardingStep"
                    />
                    <NewButton
                      :label="initializeApiError ? 'Retry' : 'Next'"
                      :tooltip="
                        !formHasRequiredValues || DEBUG_MODE
                          ? 'You must enter your credential to continue'
                          : undefined
                      "
                      tone="action"
                      :disabled="
                        !formHasRequiredValues || DEBUG_ONBOARDING_SHOW_DISABLED
                      "
                      :loading="submitOnboardingInProgress"
                      loadingText="Saving"
                      @click="submitOnboardRequest"
                    />
                  </template>
                </OnboardingStepBlock>
              </template>
              <!-- Step 3: Agent Tutorial + token -->
              <OnboardingStepBlock
                v-else-if="currentStep === OnboardingStep.SETUP_AI"
              >
                <template #header>
                  <div class="flex flex-row items-center justify-between">
                    <span>Connect the AI Agent</span>
                    <div>2/2</div>
                  </div>
                </template>
                <template #body>
                  <div
                    class="flex flex-col gap-md [&>div]:flex [&>div]:flex-col [&>div]:gap-xs"
                  >
                    <div>
                      <div class="flex flex-col">
                        <span
                          >Install
                          <NewButton
                            aria-label="claude-code-link"
                            :class="
                              clsx(
                                'underline',
                                themeClasses(
                                  'hover:text-action-500',
                                  'hover:text-action-300',
                                ),
                              )
                            "
                            tone="nostyle"
                            href="https://claude.com/product/claude-code"
                            target="_blank"
                            label="Claude Code"
                            @mousedown="
                              onboardingTracking('external_link_claude_code')
                            "
                        /></span>
                        <span
                          :class="
                            clsx(
                              themeClasses(
                                'text-neutral-800',
                                'text-neutral-300',
                              ),
                            )
                          "
                        >
                          The System Initiative AI Agent is a customized
                          installation of Claude Code.
                        </span>
                      </div>
                      <CopyableTextBlock
                        text="npm install -g @anthropic-ai/claude-code"
                        @copied="onboardingTracking('copied_install_claude')"
                      />
                    </div>
                    <div>
                      <span>
                        Clone the
                        <NewButton
                          aria-label="ai-agent-repo-main"
                          :class="
                            clsx(
                              'underline',
                              themeClasses(
                                'hover:text-action-500',
                                'hover:text-action-300',
                              ),
                            )
                          "
                          tone="nostyle"
                          href="https://github.com/systeminit/si-ai-agent"
                          target="_blank"
                          label="AI Agent repository"
                          @mousedown="
                            onboardingTracking('external_link_ai_agent_repo')
                          "
                        />
                        locally
                      </span>
                      <CopyableTextBlock
                        text="git clone https://github.com/systeminit/si-ai-agent.git"
                        @copied="onboardingTracking('copied_git_clone_ai_repo')"
                      />
                    </div>
                    <div>
                      <div class="flex flex-col">
                        <span>
                          Run the AI Agent
                          <NewButton
                            aria-label="setup-script-link"
                            :class="
                              clsx(
                                'underline',
                                themeClasses(
                                  'hover:text-action-500',
                                  'hover:text-action-300',
                                ),
                              )
                            "
                            tone="nostyle"
                            href="https://github.com/systeminit/si-ai-agent/blob/main/setup.sh"
                            target="_blank"
                            label="setup script"
                            @mousedown="
                              onboardingTracking('external_link_setup_script')
                            "
                          />
                        </span>
                        <span
                          :class="
                            clsx(
                              themeClasses(
                                'text-neutral-800',
                                'text-neutral-300',
                              ),
                            )
                          "
                        >
                          Open your terminal in the root of the repository to
                          run the script, which will connect the Agent to your
                          workspace.
                        </span>
                      </div>
                      <CopyableTextBlock
                        text="./setup.sh"
                        @copied="
                          onboardingTracking(
                            'copied_ai_setup_script_run_command',
                          )
                        "
                      />
                    </div>
                    <div>
                      <div class="flex flex-col">
                        <span>Enter your API token</span>
                        <span
                          :class="
                            clsx(
                              themeClasses(
                                'text-neutral-800',
                                'text-neutral-300',
                              ),
                            )
                          "
                        >
                          In your terminal, the setup script will ask for the
                          System Initiative API token. Paste it there. The input
                          is hidden for security. Press Enter to save and
                          proceed.
                        </span>
                      </div>
                      <ErrorMessage
                        :class="
                          clsx(
                            'rounded-sm p-xs my-xs border',
                            themeClasses(
                              'bg-warning-100 border-warning-600',
                              'bg-newhotness-warningdark border-warning-500',
                            ),
                          )
                        "
                        icon="exclamation-circle-carbon"
                        iconSize="sm"
                        tone="action"
                        variant="block"
                      >
                        <div class="text-sm">
                          We're only showing you the value of this token once.
                        </div>
                      </ErrorMessage>
                      <CopyableTextBlock
                        :text="apiToken"
                        expandable
                        @copied="onCopyAgentToken"
                      />
                    </div>
                    <div>
                      <span> Run Claude Code </span>
                      <CopyableTextBlock
                        text="claude"
                        @copied="
                          onboardingTracking('copied_run_claude_command')
                        "
                      />
                    </div>
                    <div>
                      <ErrorMessage
                        v-if="stepTwoNextDisabled"
                        tone="neutral"
                        icon="loader"
                      >
                        <div>
                          Waiting for the AI agent to start. You'll be able to
                          proceed as soon as setup is finished and Claude is
                          running.
                        </div>
                        <div>
                          If you are having trouble,
                          <NewButton
                            aria-label="schedule-meeting-stuck-agent"
                            :class="
                              clsx(
                                'underline',
                                themeClasses(
                                  'hover:text-action-500',
                                  'hover:text-action-300',
                                ),
                              )
                            "
                            tone="nostyle"
                            :href="scheduleWithUsLink"
                            target="_blank"
                            label="schedule a meeting"
                            @mousedown="
                              onboardingTracking('schedule_meeting_stuck_agent')
                            "
                          />
                          with us and we will help you out.
                        </div>
                      </ErrorMessage>
                      <ErrorMessage v-else tone="neutral" icon="check">
                        Congratulations! Your Agent is connected and you are
                        ready to start.
                      </ErrorMessage>
                    </div>
                  </div>
                </template>
                <template #footerLeft>
                  <div
                    :class="
                      clsx(
                        'text-sm leading-none',
                        themeClasses('text-neutral-800', 'text-neutral-300'),
                      )
                    "
                  >
                    Checkout our
                    <NewButton
                      aria-label="our-repo"
                      :class="
                        clsx(
                          'underline',
                          themeClasses(
                            'text-black hover:text-action-500',
                            'text-white hover:text-action-300',
                          ),
                        )
                      "
                      tone="nostyle"
                      href="https://github.com/systeminit/si-ai-agent"
                      target="_blank"
                      label="GitHub repo"
                      @mousedown="
                        onboardingTracking('checkout_our_github_repo')
                      "
                    />
                    for more guidance
                  </div>
                </template>
                <template #footerRight>
                  <NewButton
                    label="Get Started"
                    tone="action"
                    :disabled="
                      (stepTwoNextDisabled && !DEBUG_MODE) ||
                      DEBUG_ONBOARDING_SHOW_DISABLED
                    "
                    :tooltip="
                      stepTwoNextDisabled || DEBUG_MODE
                        ? 'You must set up your AI agent to continue'
                        : undefined
                    "
                    @click="onNextPageTwo"
                  />
                </template>
              </OnboardingStepBlock>
            </div>
            <div
              class="flex-1 basis-1/4 min-w-0 flex flex-col gap-lg ml-xl font-medium"
            >
              <div class="text-xl">
                <template v-if="currentStep === OnboardingStep.INITIALIZE">
                  Connect your {{ providerChoice }} account to discover, manage,
                  and automate your infrastructure.
                </template>
                <template v-else-if="currentStep === OnboardingStep.SETUP_AI">
                  Install the System Initiative AI Agent
                </template>
              </div>
              <div
                v-if="currentStep === OnboardingStep.INITIALIZE"
                class="flex flex-col gap-xs"
              >
                <CollapsingFlexItem
                  variant="onboarding"
                  open
                  @toggle="
                    onboardingTracking('toggled_why_is_a_cred_necessary')
                  "
                >
                  <template #header>
                    Why is {{ aOrAn }} {{ providerChoice }} credential
                    necessary?
                  </template>
                  <div>
                    <span
                      :class="
                        clsx(
                          themeClasses('text-neutral-800', 'text-neutral-300'),
                        )
                      "
                      >System Initiative needs access to your
                      {{ providerChoice }} account to securely discover, manage,
                      and automate your infrastructure.
                    </span>
                    <span>You make and approve all changes.</span>
                  </div>
                  <div>
                    Have questions?
                    <NewButton
                      aria-label="schedule-meeting-not-ready"
                      :class="
                        clsx(
                          'underline',
                          themeClasses(
                            'hover:text-action-500',
                            'hover:text-action-300',
                          ),
                        )
                      "
                      tone="nostyle"
                      :href="scheduleWithUsLink"
                      target="_blank"
                      label="Schedule a meeting"
                      @mousedown="
                        onboardingTracking('schedule_meeting_not_ready')
                      "
                    />
                    with us.
                  </div>
                </CollapsingFlexItem>
                <CollapsingFlexItem
                  variant="onboarding"
                  @toggle="
                    onboardingTracking('toggled_how_are_my_secrets_stored')
                  "
                >
                  <template #header>How are my secrets stored?</template>
                  <div
                    :class="
                      clsx(themeClasses('text-neutral-800', 'text-neutral-300'))
                    "
                  >
                    System Initiative secrets are secure by default. They are
                    encrypted in the browser before being transmitted over the
                    wire and encrypted at rest. All generated logs will
                    automatically redact each secret so that there are no leaks.
                    <!-- TODO - add link here when we have a good blog post to link to -->
                    <!-- <NewButton
                      aria-label="read-more-secrets"
                      :class="
                        clsx(
                          'underline',
                          themeClasses(
                            'text-black hover:text-action-500',
                            'text-white hover:text-action-300',
                          ),
                        )
                      "
                      tone="nostyle"
                      href="https://www.systeminit.com/blog/its-a-secret"
                      target="_blank"
                      label="Read more"
                      @mousedown="onboardingTracking('read-more-secrets')"
                    /> -->
                  </div>
                </CollapsingFlexItem>
                <CollapsingFlexItem
                  variant="onboarding"
                  @toggle="
                    onboardingTracking('toggled_i_dont_use_aws_or_azure')
                  "
                >
                  <template #header>
                    I don't use AWS or Azure, what should I do?
                  </template>
                  <div
                    :class="
                      clsx(themeClasses('text-neutral-800', 'text-neutral-300'))
                    "
                  >
                    We are currently adding support for additional providers.
                  </div>
                  <div>
                    <NewButton
                      aria-label="schedule-meeting-no-aws-or-azure"
                      :class="
                        clsx(
                          'underline',
                          themeClasses(
                            'hover:text-action-500',
                            'hover:text-action-300',
                          ),
                        )
                      "
                      tone="nostyle"
                      :href="scheduleWithUsLink"
                      target="_blank"
                      label="Schedule a meeting"
                      @mousedown="
                        onboardingTracking('schedule_meeting_no_aws_or_azure')
                      "
                    />
                    <span
                      :class="
                        clsx(
                          themeClasses('text-neutral-800', 'text-neutral-300'),
                        )
                      "
                    >
                      with us and tell us what you need.</span
                    >
                  </div>
                </CollapsingFlexItem>
              </div>
              <div
                v-else-if="currentStep === OnboardingStep.SETUP_AI"
                class="flex flex-col gap-xs"
              >
                <CollapsingFlexItem
                  variant="onboarding"
                  open
                  @toggle="onboardingTracking('toggled_how_does_ai_agent_work')"
                >
                  <template #header
                    >What does the System Initiative AI Agent do?</template
                  >
                  <div
                    :class="
                      clsx(themeClasses('text-neutral-800', 'text-neutral-300'))
                    "
                  >
                    The AI Agent allows you to discover resources, propose
                    changes, and understand your infrastructure using natural
                    language. Think of it like having an infrastructure expert
                    around to help out.
                  </div>
                </CollapsingFlexItem>
                <CollapsingFlexItem
                  variant="onboarding"
                  @toggle="onboardingTracking('toggled_what_am_i_installing')"
                >
                  <template #header>What am I installing?</template>
                  <div
                    :class="
                      clsx(themeClasses('text-neutral-800', 'text-neutral-300'))
                    "
                  >
                    The
                    <NewButton
                      aria-label="ai-agent-repo-right"
                      :class="
                        clsx(
                          'underline',
                          themeClasses(
                            'hover:text-action-500',
                            'hover:text-action-300',
                          ),
                        )
                      "
                      tone="nostyle"
                      href="https://github.com/systeminit/si-ai-agent"
                      target="_blank"
                      label="si-ai-agent repository"
                      @mousedown="
                        onboardingTracking('external_link_ai_agent_repo')
                      "
                    />
                    is an instance of Claude Code preconfigured to work with the
                    System Initiative MCP server. It includes both the tools
                    neccessary to work with System Inititaive and helpful
                    context for the agent.
                  </div>
                </CollapsingFlexItem>
                <CollapsingFlexItem
                  variant="onboarding"
                  @toggle="onboardingTracking('toggled_can_i_not_use_claude')"
                >
                  <template #header
                    >Can I use an Agent other than Claude?</template
                  >
                  <div
                    :class="
                      clsx(themeClasses('text-neutral-800', 'text-neutral-300'))
                    "
                  >
                    Not to get started. We have found that Claude Code is the
                    best agent for System Initiative, and we have pre-configured
                    it for a great experience. Once you have experienced it, you
                    can configure the MCP server to work with any Agent you
                    want.
                  </div>
                </CollapsingFlexItem>
              </div>
            </div>
          </template>
        </div>

        <!--  Bottom Links  -->
        <div
          class="flex flex-row flex-none w-full items-center justify-between p-sm gap-sm z-10"
        >
          <!-- Left side -->
          <div>
            <NewButton
              v-if="currentStep === OnboardingStep.PICK_PROVIDER"
              aria-label="different-provider-footer"
              :class="
                clsx(
                  'hover:underline',
                  themeClasses(
                    'text-neutral-700 hover:text-black',
                    'text-neutral-300 hover:text-white',
                  ),
                )
              "
              label="I don't use either of these providers"
              tone="nostyle"
              @click="differentProviderModal?.open()"
            />
          </div>

          <!-- Right side -->
          <div>
            <NewButton
              v-if="DEBUG_MODE"
              :icon="theme === 'dark' ? 'moon' : 'sun'"
              @click="toggleTheme"
            />
          </div>
        </div>
      </div>
    </div>
    <Modal
      ref="differentProviderModal"
      title="I don't use either of these providers"
      onboardingModal
      size="xl"
      buttonConfiguration="done"
    >
      We're working to support more providers. We can help with your special
      needs setup. Let us know what you need,
      <NewButton
        aria-label="schedule-meeting-no-aws-or-azure"
        :class="
          clsx(
            'underline',
            themeClasses('hover:text-action-500', 'hover:text-action-300'),
          )
        "
        tone="nostyle"
        :href="scheduleWithUsLink"
        target="_blank"
        label="schedule a meeting"
        @mousedown="onboardingTracking('schedule_meeting_no_aws_or_azure')"
      />
      with us.
    </Modal>
  </div>
</template>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import { computed, onMounted, reactive, ref, watch } from "vue";
import clsx from "clsx";
import {
  ErrorMessage,
  Icon,
  Modal,
  NewButton,
  themeClasses,
  userOverrideTheme,
  useTheme,
} from "@si/vue-lib/design-system";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import * as _ from "lodash-es";
import { encryptMessage } from "@/utils/messageEncryption";
import { componentTypes, routes, useApi } from "@/newhotness/api_composables";
import { useContext } from "@/newhotness/logic_composables/context";
import { trackEvent } from "@/utils/tracking";
import OnboardingStepBlock from "@/newhotness/OnboardingStepBlock.vue";
import { useAuthStore } from "@/store/auth.store";
import CopyableTextBlock from "./CopyableTextBlock.vue";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import OnboardingProviderTile, { Provider } from "./OnboardingProviderTile.vue";
import RequiredAsterisk from "./RequiredAsterisk.vue";

const { theme } = useTheme();

function toggleTheme() {
  userOverrideTheme.value = theme.value === "dark" ? "light" : "dark";
}

const differentProviderModal = ref<InstanceType<typeof Modal>>();

const scheduleWithUsLink =
  "https://calendly.com/d/cw8r-6rq-b3n/share-your-use-case-with-system-initiative";
// const scheduleADemoLink =
//   "https://calendly.com/d/cns7-v2b-jkz/system-initiative-demo";

const ctx = useContext();

const authStore = useAuthStore();

const hasUsedAiAgent = computed(
  () => authStore.userWorkspaceFlags.executedAgent ?? false,
);

/// STARTUP LOGIC
const currentStep = ref<OnboardingStep>(
  DEBUG_MODE ? DEBUG_ONBOARDING_START : OnboardingStep.PICK_PROVIDER,
);
const incrementOnboardingStep = () => {
  currentStep.value = Math.min(
    OnboardingStep.SETUP_AI,
    currentStep.value + 1,
  ) as OnboardingStep;
};
const decrementOnboardingStep = () => {
  currentStep.value = Math.max(
    OnboardingStep.INITIALIZE,
    currentStep.value - 1,
  ) as OnboardingStep;
  if (currentStep.value === OnboardingStep.PICK_PROVIDER) {
    providerChoice.value = undefined;
  }
};

const onNextPageTwo = () => {
  onboardingTracking("finish_step_2_connect_your_ai_agent");
  closeOnboarding();
};

// CREDENTIAL
const credentialName = ref(`My ${DEBUG_PROVIDER_CHOICE} Credential`);

const secretFormFieldsAWS = reactive({
  "AWS access key ID": {
    ref: "",
    type: "password",
    required: true,
  },
  "AWS secret access key": {
    ref: "",
    type: "password",
    required: true,
  },
  "AWS session token": {
    ref: "",
    type: "password",
    required: false,
  },
});

const secretFormFieldsAzure = reactive({
  "Client ID": {
    ref: "",
    type: "password",
    required: true,
  },
  "Client Secret": {
    ref: "",
    type: "password",
    required: true,
  },
  "Tenant ID": {
    ref: "",
    type: "password",
    required: true,
  },
});

const formFieldsForProvider = computed(() => {
  if (providerChoice.value === "AWS") {
    return secretFormFieldsAWS;
  } else if (providerChoice.value === "Azure") {
    return secretFormFieldsAzure;
  } else {
    return undefined;
  }
});

const formHasRequiredValues = computed(() => {
  const formFields = formFieldsForProvider.value;
  if (!formFields) return false;

  const credentialFieldsFilled =
    !Object.values(formFields).some((f) => f.required && f.ref === "") &&
    credentialName.value !== "";

  // For Azure, also check subscription ID
  if (providerChoice.value === "Azure") {
    return (
      (credentialFieldsFilled && azureSubscriptionId.value !== "") || DEBUG_MODE
    );
  }

  return credentialFieldsFilled || DEBUG_MODE;
});

type SecretFormField = {
  ref: string;
  type: string;
  required: boolean;
};

/// Match env var block to form fields
const tryMatchOnPaste = (ev: ClipboardEvent) => {
  const text = ev.clipboardData?.getData("text/plain");
  if (!text) return;

  const formFields = formFieldsForProvider.value;
  if (!formFields) return;

  const valuesFromInput = text.split("\n").reduce((acc, e) => {
    const [_, key, value] = e.match(/export\s+(.*)="(.*)"/) ?? [];
    if (!key || !value) return acc;
    acc[key] = value;
    return acc;
  }, {} as Record<string, string>);

  let matchedAValue = false;
  // Loop through form fields and try to match the value keys to the titles
  for (const [formKey, formValue] of Object.entries(
    formFields as Record<string, SecretFormField>,
  )) {
    const formattedKey = formKey.replaceAll(" ", "_").toUpperCase();
    const matchedField = valuesFromInput[formattedKey];

    if (!matchedField) continue;
    matchedAValue = true;

    formValue.ref = matchedField;
  }

  // If we didn't match a value, just proceed with the paste
  if (!matchedAValue) return;

  ev.preventDefault();
};

const toggleVisibility = (field: SecretFormField) => {
  field.type = field.type === "password" ? "text" : "password";
};
const someFieldsVisible = computed(() => {
  const formFields = formFieldsForProvider.value;
  if (!formFields) return false;

  return Object.values(formFields).some((field) => field.type !== "password");
});
const toggleAll = () => {
  const formFields = formFieldsForProvider.value;
  if (!formFields) return;

  if (someFieldsVisible.value) {
    Object.values(formFields).forEach((field) => {
      field.type = "password";
    });
  } else {
    Object.values(formFields).forEach((field) => {
      field.type = "text";
    });
  }
};

// PROVIDER
const providerChoice = ref<Provider | undefined>(
  DEBUG_MODE ? DEBUG_PROVIDER_CHOICE : undefined,
);
const pickProvider = (provider: Provider) => {
  providerChoice.value = provider;
  credentialName.value = `My ${provider} Credential`;
  onboardingTracking(`picked_provider_${provider.toLowerCase()}`);
  incrementOnboardingStep();
};
const aOrAn = computed(() =>
  providerChoice.value?.match("^[aieouAIEOU].*") ? "an" : "a",
);

// REGION
const awsRegion = ref("us-east-1");
const awsRegions = [
  { title: "US East (N. Virginia)", value: "us-east-1", onPicker: true },
  { title: "US West (Oregon)", value: "us-west-2", onPicker: true },
  { title: "US West (N. California)", value: "us-west-1", onPicker: true },
  { title: "Europe (Ireland)", value: "eu-west-1" },
  { title: "Europe (Frankfurt)", value: "eu-central-1" },
  { title: "Asia Pacific (Singapore)", value: "ap-southeast-1" },
  { title: "Asia Pacific (Tokyo)", value: "ap-northeast-1" },
  { title: "Asia Pacific (Sydney)", value: "ap-southeast-2" },
  { title: "US East (Ohio)", value: "us-east-2" },
  { title: "Europe (London)", value: "eu-west-2" },
  { title: "Africa (Cape Town)", value: "af-south-1" },
  { title: "Asia Pacific (Hong Kong)", value: "ap-east-1" },
  { title: "Asia Pacific (Taipei)", value: "ap-east-2" },
  { title: "Asia Pacific (Jakarta)", value: "ap-southeast-3" },
  { title: "Asia Pacific (Melbourne)", value: "ap-southeast-4" },
  { title: "Asia Pacific (Malaysia)", value: "ap-southeast-5" },
  { title: "Asia Pacific (Thailand)", value: "ap-southeast-7" },
  { title: "Asia Pacific (Mumbai)", value: "ap-south-1" },
  { title: "Asia Pacific (Hyderabad)", value: "ap-south-2" },
  { title: "Asia Pacific (Seoul)", value: "ap-northeast-2" },
  { title: "Asia Pacific (Osaka)", value: "ap-northeast-3" },
  { title: "Canada (Central)", value: "ca-central-1" },
  { title: "Canada West (Calgary)", value: "ca-west-1" },
  { title: "Europe (Zurich)", value: "eu-central-2" },
  { title: "Europe (Paris)", value: "eu-west-3" },
  { title: "Europe (Milan)", value: "eu-south-1" },
  { title: "Europe (Spain)", value: "eu-south-2" },
  { title: "Europe (Stockholm)", value: "eu-north-1" },
  { title: "Israel (Tel Aviv)", value: "il-central-1" },
  { title: "Middle East (Bahrain)", value: "me-south-1" },
  { title: "Middle East (UAE)", value: "me-central-1" },
  { title: "Mexico (Central)", value: "mx-central-1" },
  { title: "South America (Sao Paulo)", value: "sa-east-1" },
  { title: "AWS GovCloud (US-East)", value: "us-gov-east-1" },
  { title: "AWS GovCloud (US-West)", value: "us-gov-west-1" },
];
const pickerRegions = awsRegions.filter((r) => r.onPicker);
const selectRegion = (regionValue: string) => {
  awsRegion.value = regionValue;
};
watch(awsRegion, (regionValue) => {
  onboardingTracking(`selected_region_${regionValue.replace(/-/g, "_")}`);
});

// LOCATION
const azureLocation = ref("eastus");
const azureSubscriptionId = ref("");
const azureSubscriptionIdFieldType = ref("password");
const azureLocations = [
  { title: "East US", value: "eastus", onPicker: true },
  { title: "South Central US", value: "southcentralus" },
  { title: "West US 2", value: "westus2", onPicker: true },
  { title: "West US 3", value: "westus3" },
  { title: "Australia East", value: "australiaeast" },
  { title: "Southeast Asia", value: "southeastasia" },
  { title: "North Europe", value: "northeurope" },
  { title: "Sweden Central", value: "swedencentral" },
  { title: "UK South", value: "uksouth", onPicker: true },
  { title: "West Europe", value: "westeurope" },
  { title: "Central US", value: "centralus" },
  { title: "South Africa North", value: "southafricanorth" },
  { title: "Central India", value: "centralindia" },
  { title: "East Asia", value: "eastasia" },
  { title: "Japan East", value: "japaneast" },
  { title: "Korea Central", value: "koreacentral" },
  { title: "Canada Central", value: "canadacentral" },
  { title: "France Central", value: "francecentral" },
  { title: "Germany West Central", value: "germanywestcentral" },
  { title: "Italy North", value: "italynorth" },
  { title: "Norway East", value: "norwayeast" },
  { title: "Poland Central", value: "polandcentral" },
  { title: "Spain Central", value: "spaincentral" },
  { title: "Switzerland North", value: "switzerlandnorth" },
  { title: "Mexico Central", value: "mexicocentral" },
  { title: "UAE North", value: "uaenorth" },
  { title: "Brazil South", value: "brazilsouth" },
  { title: "Israel Central", value: "israelcentral" },
  { title: "Qatar Central", value: "qatarcentral" },
];
const pickerLocations = azureLocations.filter((l) => l.onPicker);
const selectLocation = (locationValue: string) => {
  azureLocation.value = locationValue;
};
watch(azureLocation, (locationValue) => {
  onboardingTracking(`selected_location_${locationValue.replace(/-/g, "_")}`);
});
const toggleSubscriptionIdVisibility = () => {
  azureSubscriptionIdFieldType.value =
    azureSubscriptionIdFieldType.value === "password" ? "text" : "password";
};

const initializeApi = useApi();
// TODO Figure out where to put this
const initializeApiError = ref<string | null>(null);
const submittedOnboardRequest = ref(false);
const keyApi = useApi();

const initializeRequestSentAndSuccessful = computed(() => {
  return (
    submittedOnboardRequest.value &&
    !initializeApi.inFlight.value &&
    !initializeApiError.value
  );
});

const submitOnboardingInProgress = ref(false);

const submitOnboardRequest = async () => {
  // Can't submit an onboard request without a selected provider!
  if (!providerChoice.value) return;

  // Tracking
  onboardingTracking(
    `finish_step_1_submit_${providerChoice.value.toLowerCase()}_info`,
  );

  // Disable button
  submitOnboardingInProgress.value = true;

  if (DEBUG_MODE) {
    // debug mode skips creating credentials
    incrementOnboardingStep();
    return;
  }

  // Encrypt secret
  const callApi = keyApi.endpoint<componentTypes.PublicKey>(
    routes.GetPublicKey,
    { id: "00000000000000000000000000" }, // TODO Remove component id from this endpoint's path, it's not needed
  );
  const resp = await callApi.get();
  const publicKey = resp.data;

  let credValue;

  // Format cred values for encryption
  if (providerChoice.value === "AWS") {
    credValue = (
      [
        ["SessionToken", secretFormFieldsAWS["AWS session token"].ref],
        ["AccessKeyId", secretFormFieldsAWS["AWS access key ID"].ref],
        ["SecretAccessKey", secretFormFieldsAWS["AWS secret access key"].ref],
      ].filter(([_, value]) => value !== "") as [string, string][]
    ) // Remove empty values
      .reduce<{ [key: string]: string }>((acc, [key, value]) => {
        // make the pairs array into an object
        acc[key] = value;
        return acc;
      }, {});
  } else if (providerChoice.value === "Azure") {
    credValue = (
      [
        ["ClientId", secretFormFieldsAzure["Client ID"].ref],
        ["ClientSecret", secretFormFieldsAzure["Client Secret"].ref],
        ["TenantId", secretFormFieldsAzure["Tenant ID"].ref],
      ].filter(([_, value]) => value !== "") as [string, string][]
    ) // Remove empty values
      .reduce<{ [key: string]: string }>((acc, [key, value]) => {
        // make the pairs array into an object
        acc[key] = value;
        return acc;
      }, {});
  } else {
    // No valid provider! This case should be impossible.
    return;
  }

  const crypted = await encryptMessage(credValue, publicKey);

  submittedOnboardRequest.value = true;
  const call = initializeApi.endpoint(routes.ChangeSetInitializeAndApply);

  // Build provider object based on selected provider
  let provider;
  if (providerChoice.value === "AWS") {
    provider = {
      type: "Aws",
      region: awsRegion.value,
    };
  } else if (providerChoice.value === "Azure") {
    provider = {
      type: "Azure",
      location: azureLocation.value,
      subscriptionId: azureSubscriptionId.value,
    };
  }

  const { errorMessage } = await call.post({
    credential: {
      name: credentialName.value,
      crypted,
      keyPairPk: publicKey.pk,
      version: componentTypes.SecretVersion.V1,
      algorithm: componentTypes.SecretAlgorithm.Sealedbox,
    },
    provider,
  });

  submitOnboardingInProgress.value = false;

  if (errorMessage) {
    initializeApiError.value = errorMessage;
  } else {
    incrementOnboardingStep();
  }
};

// AI CONFIGURATION
const generateTokenApi = useApi();
const apiToken = ref();
const generateToken = async () => {
  const workspacePk = ctx.workspacePk.value;
  const userPk = ctx.user?.pk;

  const apiTokenSessionStorageKey = `si-api-token-${workspacePk}-${userPk}`;

  const storedToken = sessionStorage.getItem(apiTokenSessionStorageKey);
  if (storedToken) {
    apiToken.value = storedToken;
    return;
  }

  const callApi = generateTokenApi.endpoint<{ token: string }>(
    routes.GenerateApiToken,
  );
  const {
    req: { data },
  } = await callApi.post({
    name: "Onboarding Key",
    expiration: "1y",
  });

  const token = data.token;

  if (!token) {
    // TODO deal with errors on API Token generation
    // eslint-disable-next-line no-console
    console.error("No token generated");
    return;
  }

  sessionStorage.setItem(apiTokenSessionStorageKey, token);
  apiToken.value = token;
};

onMounted(() => {
  generateToken();
});

const dismissOnboardingApi = useApi();
const closeOnboarding = async (fast = false) => {
  const userPk = ctx.user?.pk;
  if (!userPk) return;

  const call = dismissOnboardingApi.endpoint(routes.DismissOnboarding, {
    userPk,
  });

  if (fast) {
    call.post({});
  } else {
    await call.post({});
  }

  emit("completed");
};

const agentTokenCopied = ref(false);
const onCopyAgentToken = () => {
  agentTokenCopied.value = true;
  onboardingTracking("ai_token_copied");
};

const stepTwoNextDisabled = computed(
  () => !initializeRequestSentAndSuccessful.value || !hasUsedAiAgent.value,
);

const emit = defineEmits<{
  (e: "completed"): void;
}>();

const onboardingTracking = (eventName: string) => {
  if (DEBUG_MODE) {
    // eslint-disable-next-line no-console
    console.log(`DEBUG MODE TRACKING NOT FIRED: '${eventName}'`);
  } else {
    trackEvent(`onboarding_${eventName}`);
  }
};
</script>

<style lang="css" scoped>
.steps {
  grid-template-columns: 32px 1fr;
  grid-template-rows: auto auto;
  grid-template-areas:
    "number primary"
    "numberline secondary";
}

.number {
  grid-area: number;
}

.numberline {
  grid-area: numberline;
}

.primary {
  grid-area: primary;
}

.secondary {
  grid-area: secondary;
}

.bg-onboardingcircle1 {
  background: radial-gradient(
    50% 50% at 50% 50%,
    rgba(240, 115, 0, 0.6) 0%,
    rgba(124, 72, 24, 0) 100%
  );
  opacity: 0.3;
  background-blend-mode: color;
}

.bg-onboardingcircle2 {
  background: radial-gradient(
    50% 50% at 54.57% 49.79%,
    #50e6e6 0%,
    rgba(45, 128, 128, 0) 100%
  );
  opacity: 0.3;
  background-blend-mode: color;
}
</style>
