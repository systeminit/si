<template>
  <Panel
    initialPanelType="workflow"
    :panelIndex="panelIndex"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
    :isVisible="isVisible"
    :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
    v-on="$listeners"
  >
    <template v-slot:menuButtons>
      <div class="flex">
        <SiSelect
          size="xs"
          id="workflowPanelSelect"
          name="workflowPanelSelect"
          :options="workflowList"
          v-model="selectedWorkflow"
          class="pl-1"
        />
      </div>
      <button
        class="pl-1 focus:outline-none disabled:opacity-30"
        :disabled="!selectedWorkflow"
        @click="runThisWorkflow()"
      >
        <PlayCircleIcon size="1.1x" />
      </button>
      <div class="flex flex-row items-center pl-1">
        <div class="flex">
          <input type="checkbox" name="dryRun" v-model="dryRun" />
        </div>
        <div class="flex pl-1 text-xs">
          Dry
        </div>
      </div>
    </template>
    <template v-slot:content>
      <div class="flex flex-col w-full overflow-hidden">
        <div
          class="relative flex flex-row items-center w-full h-10 pt-2 pb-2 pl-6 pr-6 text-base text-white align-middle property-section-bg-color"
        >
          <div class="flex text-lg">
            {{ workflowLabel }}
          </div>
          <div class="flex justify-end flex-grow text-xs text-gray-500">
            <template v-if="dryRun">
              Dry Run
            </template>
          </div>
        </div>
        <div class="flex flex-col mt-2 overflow-auto">
          <div
            class="flex flex-col w-full mb-2"
            v-for="run in workflowRuns"
            :key="run.id"
          >
            <div
              class="flex flex-row w-full"
              :class="workflowRunTitleClasses(run)"
            >
              <div class="flex">{{ run.startTime }} {{ run.state }}</div>
              <div class="flex justify-end pl-1 text-xs" v-if="run.dryRun">
                (dry run)
              </div>
              <div class="flex justify-end flex-grow pr-4">
                <button @click.prevent="toggleActions(run.id)">
                  <MoreHorizontalIcon />
                </button>
              </div>
            </div>
            <div
              class="flex flex-col w-full pl-4 pr-4"
              v-if="visibleActions[run.id]"
            >
              <div
                class="flex flex-col pt-2"
                v-for="action in run.actions"
                :key="action.id"
              >
                <div
                  class="flex flex-row w-full"
                  :class="actionTitleClasses(action)"
                >
                  <div class="flex flex-row">
                    {{ action.entityName }} {{ action.name }} {{ action.state }}
                  </div>
                  <div class="flex justify-end flex-grow pr-4">
                    <button
                      @click.prevent="toggleSpecificActions(run.id, action.id)"
                    >
                      <MoreHorizontalIcon />
                    </button>
                  </div>
                </div>
                <div
                  class="flex flex-col w-full pl-4"
                  v-if="visibleSpecificAction(run.id, action.id)"
                >
                  <div class="flex flex-row mt-2">
                    <div class="flex justify-end w-20">
                      startTime:
                    </div>
                    <div class="flex flex-grow pl-2">
                      {{ action.startTime }}
                    </div>
                  </div>
                  <div class="flex flex-row mt-2">
                    <div class="flex justify-end w-20">
                      endTime:
                    </div>
                    <div class="flex flex-grow pl-2">
                      {{ action.endTime }}
                    </div>
                  </div>
                  <div class="flex flex-row mt-2" v-if="action.output">
                    <div class="flex justify-end w-20">
                      output:
                    </div>
                    <div class="flex flex-grow pl-2">
                      <CodeMirror
                        :value="action.output"
                        readOnly
                        lineWrapping
                        noHighlight
                      />
                    </div>
                  </div>
                  <div class="flex flex-row mt-2" v-if="action.error">
                    <div class="flex justify-end w-20">
                      error:
                    </div>
                    <div class="flex flex-grow pl-2">
                      <CodeMirror
                        :value="action.error"
                        readOnly
                        lineWrapping
                        noHighlight
                      />
                    </div>
                  </div>
                  <div
                    class="flex flex-row mt-2"
                    v-if="action.resourceDiff.length"
                  >
                    <div class="flex justify-end w-20">
                      diff:
                    </div>
                    <div class="flex flex-col flex-grow pl-2">
                      <div
                        class="flex flex-row"
                        v-for="(diff, index) in action.resourceDiff"
                        :key="index"
                      >
                        <div
                          v-if="diff.edit"
                          class="flex flex-col w-full border-b-2"
                        >
                          <div class="flex flex-row w-20">
                            Edit: {{ diff.edit.path.join("/") }}
                          </div>
                          <div class="flex flex-row flex-grow">
                            <div class="flex flex-col p-1">
                              <div class="flex bg-gray-800">
                                Before
                              </div>
                              <div class="flex">
                                {{ diff.edit.before }}
                              </div>
                            </div>
                            <div class="flex flex-col p-1">
                              <div class="flex bg-gray-800">
                                After
                              </div>
                              <div class="flex">
                                {{ diff.edit.after }}
                              </div>
                            </div>
                          </div>
                        </div>
                        <div
                          v-else-if="diff.add"
                          class="flex flex-col w-full border-b-2"
                        >
                          <div class="flex flex-row w-20">
                            Add: {{ diff.add.path.join("/") }}
                          </div>
                          <div class="flex flex-col flex-grow">
                            <div class="flex flex-col p-1">
                              <div class="flex bg-gray-800">
                                After:
                              </div>
                              <div class="flex">
                                {{ diff.add.after }}
                              </div>
                            </div>
                          </div>
                        </div>
                        <div
                          v-else-if="diff.remove"
                          class="flex flex-col w-full border-b-2"
                        >
                          <div class="flex flex-row w-20">
                            Remove: {{ diff.remove.path.join("/") }}
                          </div>
                          <div class="flex flex-col flex-grow">
                            <div class="flex flex-col p-1">
                              <div class="flex bg-gray-800">
                                Before:
                              </div>
                              <div class="flex">
                                {{ diff.remove.before }}
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </Panel>
</template>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>

<script lang="ts">
import Vue from "vue";
import _ from "lodash";

import { workflows } from "si-registry";
import {
  Resource,
  ResourceHealth,
  ResourceStatus,
} from "@/api/sdf/model/resource";

import Panel from "@/molecules/Panel.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import CodeMirror from "@/molecules/CodeMirror.vue";
import { PlayCircleIcon, MoreHorizontalIcon } from "vue-feather-icons";

import { WorkflowRunState, WorkflowRun } from "@/api/sdf/model/workflow";
import { ActionState, Action } from "@/api/sdf/model/action";
import { runWorkflow, system$, applicationId$ } from "@/observables";
import { switchMap } from "rxjs/operators";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface WorkflowRunWithActions extends WorkflowRun {
  actions: Actions[];
}

interface Data {
  workflowList: {
    label: string;
    value: string;
  }[];
  selectedWorkflow: string;
  dryRun: boolean;
  workflowRuns: WorkflowRunWithActions[];
  visibleActions: {
    [runId: string]: boolean;
  };
  visibleSpecificActions: {
    [key: string]: boolean;
  };
}

export default Vue.extend({
  name: "WorkflowPanel",
  components: {
    Panel,
    SiSelect,
    PlayCircleIcon,
    MoreHorizontalIcon,
    CodeMirror,
  },
  props: {
    panelIndex: Number,
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
    isVisible: Boolean,
    isMaximizedContainerEnabled: Boolean,
  },
  data(): Data {
    const workflowList: Data["workflowList"] = _.flatMap(workflows, w => {
      return { label: w.title, value: w.name };
    });
    workflowList.unshift({ label: "", value: "" });

    const workflowRuns: Data["workflowRuns"] = [
      {
        id: "1",
        startTime: Date.now().toString(),
        endTime: Date.now().toString(),
        state: WorkflowRunState.Success,
        dryRun: false,
        actions: [
          {
            id: "1",
            workflowRunId: "1",
            dryRun: false,
            name: "deploy",
            startTime: Date.now().toString(),
            endTime: Date.now().toString(),
            entityId: "xx",
            entityName: "poop",
            state: ActionState.Success,
            output: "there is no doubt you're in my heart now",
            resourceDiff: [
              {
                edit: {
                  path: ["health"],
                  before: ResourceHealth.Error,
                  after: ResourceHealth.Ok,
                },
              },
              {
                add: {
                  path: ["health"],
                  after: ResourceHealth.Ok,
                },
              },
              {
                remove: {
                  path: ["health"],
                  before: ResourceHealth.Ok,
                },
              },
            ],
            resource: {
              id: "w",
              unixTimestamp: Date.now().toString(),
              timestamp: Date.now().toString(),
              state: { wagon: "wheel" },
              status: ResourceStatus.Created,
              health: ResourceHealth.Ok,
              systemId: "p",
              nodeId: "n",
              entityId: "e",
              siStorable: {
                typeName: "resource",
                objectId: "r",
                billingAccountId: "b",
                organizationId: "o",
                workspaceId: "w",
                tenantIds: ["b", "o", "w"],
                deleted: false,
              },
            },
          },
        ],
      },
      {
        id: "2",
        startTime: (Date.now() - 10).toString(),
        endTime: (Date.now() + 10).toString(),
        state: WorkflowRunState.Failure,
        dryRun: false,
        actions: [
          {
            id: "1",
            workflowRunId: "2",
            startTime: Date.now().toString(),
            endTime: Date.now().toString(),
            dryRun: false,
            name: "deploy",
            entityId: "xx",
            entityName: "poop",
            state: ActionState.Failure,
            error: "just a little patience",
            resourceDiff: [],
            resource: {
              id: "w",
              unixTimestamp: Date.now().toString(),
              timestamp: Date.now().toString(),
              state: { wagon: "wheel" },
              status: ResourceStatus.Created,
              health: ResourceHealth.Ok,
              systemId: "p",
              nodeId: "n",
              entityId: "e",
              siStorable: {
                typeName: "resource",
                objectId: "r",
                billingAccountId: "b",
                organizationId: "o",
                workspaceId: "w",
                tenantIds: ["b", "o", "w"],
                deleted: false,
              },
            },
          },
        ],
      },
      {
        id: "3",
        startTime: (Date.now() - 10).toString(),
        endTime: (Date.now() + 10).toString(),
        state: WorkflowRunState.Running,
        dryRun: true,
        actions: [
          {
            id: "1",
            workflowRunId: "3",
            startTime: Date.now().toString(),
            endTime: Date.now().toString(),
            dryRun: true,
            name: "deploy",
            entityId: "xx",
            entityName: "poop",
            state: ActionState.Failure,
            error: "just a little patience",
            resourceDiff: [],
            resource: {
              id: "w",
              unixTimestamp: Date.now().toString(),
              timestamp: Date.now().toString(),
              state: { wagon: "wheel" },
              status: ResourceStatus.Created,
              health: ResourceHealth.Ok,
              systemId: "p",
              nodeId: "n",
              entityId: "e",
              siStorable: {
                typeName: "resource",
                objectId: "r",
                billingAccountId: "b",
                organizationId: "o",
                workspaceId: "w",
                tenantIds: ["b", "o", "w"],
                deleted: false,
              },
            },
          },
        ],
      },
    ];

    return {
      workflowList,
      selectedWorkflow: "",
      workflowRuns,
      dryRun: false,
      visibleActions: {},
      visibleSpecificActions: {},
    };
  },
  computed: {
    workflowLabel(): string {
      if (workflows[this.selectedWorkflow]) {
        return workflows[this.selectedWorkflow].title;
      } else {
        return "No Workflow Selected!";
      }
    },
  },
  subscriptions() {
    return {
      system: system$,
      applicationId: applicationId$,
    };
  },
  methods: {
    runThisWorkflow(): void {
      // @ts-ignore
      if (this.selectedWorkflow && this.system && this.applicationId) {
        runWorkflow({
          workflowName: this.selectedWorkflow,
          args: {
            // @ts-ignore
            system: this.system.id,
            // @ts-ignore
            application: this.applicationId,
          },
        }).subscribe(reply => {
          // going to have to combine them!
          if (reply.error && reply.error.code != 42) {
            emitEditorErrorMessage(reply.error.message);
          }
        });
      }
    },
    toggleSpecificActions(runId: string, actionId: string): void {
      const newKey = `${runId}-${actionId}`;
      if (this.visibleSpecificActions[newKey]) {
        Vue.set(this.visibleSpecificActions, newKey, false);
      } else {
        Vue.set(this.visibleSpecificActions, newKey, true);
      }
    },
    visibleSpecificAction(runId: string, actionId: string): boolean {
      const newKey = `${runId}-${actionId}`;
      if (this.visibleSpecificActions[newKey]) {
        return this.visibleSpecificActions[newKey];
      } else {
        return false;
      }
    },
    toggleActions(runId: string): void {
      if (this.visibleActions[runId]) {
        Vue.set(this.visibleActions, runId, false);
      } else {
        Vue.set(this.visibleActions, runId, true);
      }
    },
    workflowRunTitleClasses(run: WorkflowRun): Record<string, any> {
      const classes: Record<string, any> = {};
      if (run.state == WorkflowRunState.Running) {
        classes["bg-indigo-700"] = true;
      } else if (run.state == WorkflowRunState.Success) {
        classes["bg-green-700"] = true;
      } else if (run.state == WorkflowRunState.Failure) {
        classes["bg-red-700"] = true;
      } else {
        classes["bg-gray-700"] = true;
      }
      return classes;
    },
    actionTitleClasses(action: Action): Record<string, any> {
      const classes: Record<string, any> = {};
      if (action.state == ActionState.Running) {
        classes["bg-indigo-700"] = true;
      } else if (action.state == ActionState.Success) {
        classes["bg-green-700"] = true;
      } else if (action.state == ActionState.Failure) {
        classes["bg-red-700"] = true;
      } else {
        classes["bg-gray-700"] = true;
      }
      return classes;
    },
  },
});
</script>
