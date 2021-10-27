<template>
  <div class="flex flex-col w-full" v-if="entity">
    <div
      class="flex flex-row items-center justify-between w-full h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} actions
      </div>

      <div class="ml-2 text-base">
        <BoxIcon size="1x" :class="resourceHealthStatus" />
      </div>
    </div>
    <div
      class="relative flex flex-row items-center justify-between pt-2 pb-2 pl-6 pr-6 text-base text-white bg-black"
    >
      <div class="flex flex-row">
        <div class="flex flex-row">
          <SiSelect
            size="xs"
            id="actionPanelSelect"
            name="actionPanelSelect"
            :options="actionList"
            v-model="selectedAction"
            class="pl-1"
          />
        </div>

        <button
          class="pl-1 focus:outline-none disabled:opacity-30"
          :disabled="!selectedAction || changeSet"
          @click="runThisAction()"
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
      </div>

      <div class="flex flex-row">
        <button
          class="focus:outline-none disabled:opacity-30"
          :class="successesFilterClasses()"
          @click="toggleShowSucceeded()"
        >
          <CheckCircleIcon size="1x" />
        </button>

        <button
          class="ml-2 focus:outline-none disabled:opacity-30"
          :class="failuresFilterClasses()"
          @click="toggleShowFailed()"
        >
          <AlertCircleIcon size="1x" />
        </button>
      </div>
    </div>

    <div class="w-full h-full workflow-runs">
      <div class="flex flex-col w-full mt-6">
        <WorkflowRun
          v-for="(data, index) in workflowRuns"
          class="mx-4 drop-shadow-xl"
          :key="index"
          :workflowRun="data.workflowRun"
          :workflowSteps="data.steps"
          v-show="showRun(data.workflowRun)"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import { RegistryEntry, registry } from "si-registry";
import {
  WorkflowDal,
  IListActionReplySuccess,
  IListActionRequest,
} from "@/api/sdf/dal/workflowDal";

// TODO: Get the list of workflows run for this entity; if they select one, filter. If they hit play, run.
import SiSelect from "@/atoms/SiSelect.vue";
import {
  PlayCircleIcon,
  CheckCircleIcon,
  AlertCircleIcon,
  BoxIcon,
} from "vue-feather-icons";
import { ILabelListItem } from "@/api/sdf/dal";
import {
  system$,
  workspace$,
  changeSet$,
  editMode$,
  workflowRuns$,
  workflowRunSteps$,
  workflowRunStepEntities$,
  refreshActivitySummary$,
} from "@/observables";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { combineLatest } from "rxjs";
import { tap, pluck, map } from "rxjs/operators";

import WorkflowRun from "@/molecules/WorkflowRun.vue";
import { Resource, ResourceHealth } from "@/api/sdf/model/resource";

import {
  WorkflowRun as WorkflowRunType,
  WorkflowRunState,
} from "@/api/sdf/model/workflow";

interface Data {
  dryRun: boolean;
  selectedAction: string;
  workflowRuns: IListActionReplySuccess["workflowRuns"];
  isSucceededVisible: boolean;
  isFailedVisible: boolean;
}

// The critical data are the 'worfklowRun', 'workflowRunStep', and 'workflowRunStepEntity'.

export default Vue.extend({
  name: "ActionViewer",
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  components: {
    PlayCircleIcon,
    SiSelect,
    WorkflowRun,
    CheckCircleIcon,
    AlertCircleIcon,
    BoxIcon,
  },
  data(): Data {
    return {
      dryRun: false,
      selectedAction: "",
      workflowRuns: [],
      isSucceededVisible: true,
      isFailedVisible: true,
    };
  },
  computed: {
    schema(): RegistryEntry | null {
      if (registry[this.entity.entityType]) {
        return registry[this.entity.entityType];
      } else {
        return null;
      }
    },
    actionList(): ILabelListItem[] {
      let response = [{ label: "", value: "" }];
      if (this.schema?.actions) {
        for (const action of this.schema.actions) {
          response.push({ label: action.name, value: action.name });
        }
      }
      return response;
    },
    resourceHealthStatus(): Record<string, any> {
      let style: Record<string, any> = {};

      if (this.resource) {
        if (this.resource.health == ResourceHealth.Ok) {
          style["ok"] = true;
        } else if (this.resource.health == ResourceHealth.Warning) {
          style["warning"] = true;
        } else if (this.resource.health == ResourceHealth.Error) {
          style["error"] = true;
        } else if (this.resource.health == ResourceHealth.Unknown) {
          style["unknown"] = true;
        } else {
          style["unknown"] = true;
        }
      } else {
        style["unknown"] = true;
      }
      return style;
    },
  },
  subscriptions: function(this: any): Record<string, any> {
    return {
      editMode: editMode$,
      changeSet: changeSet$,
      system: system$,
      workspace: workspace$,
      workflowRunsUpdate: workflowRuns$.pipe(
        tap(workflowRun => {
          if (workflowRun.ctx.entity?.id == this.entity.id) {
            for (let x = 0; x < this.workflowRuns.length; x++) {
              if (this.workflowRuns[x].workflowRun.id == workflowRun.id) {
                let wfr = { workflowRun, steps: this.workflowRuns[x].steps };
                Vue.set(this.workflowRuns, x, wfr);
                return;
              }
            }
            this.workflowRuns.unshift({ workflowRun, steps: [] });
          }
        }),
      ),
      workflowRunStepsUpdate: workflowRunSteps$.pipe(
        tap(workflowRunStep => {
          for (const run of this
            .workflowRuns as IListActionReplySuccess["workflowRuns"]) {
            if (run.workflowRun.id == workflowRunStep.workflowRunId) {
              for (const step of run.steps) {
                if (step.step.id == workflowRunStep.id) {
                  step.step = workflowRunStep;
                  return;
                }
              }
              run.steps.push({ step: workflowRunStep, stepEntities: [] });
              return;
            }
          }
        }),
      ),
      workflowRunStepEntitiesUpdate: workflowRunStepEntities$.pipe(
        tap(workflowRunStepEntity => {
          for (const run of this
            .workflowRuns as IListActionReplySuccess["workflowRuns"]) {
            if (run.workflowRun.id == workflowRunStepEntity.workflowRunId) {
              for (const step of run.steps) {
                if (step.step.id == workflowRunStepEntity.workflowRunStepId) {
                  for (let x = 0; x < step.stepEntities.length; x++) {
                    if (step.stepEntities[x].id == workflowRunStepEntity.id) {
                      Vue.set(step.stepEntities, x, workflowRunStepEntity);
                      return;
                    }
                  }
                  step.stepEntities.push(workflowRunStepEntity);
                  return;
                }
              }
            }
          }
        }),
      ),
      actionWorkflows: combineLatest(
        system$,
        workspace$,
        this.$watchAsObservable("entity", { immediate: true }).pipe(
          pluck("newValue"),
          tap(_entity => {
            this.workflowRuns = []; // This might want to set a loading indicator!
          }),
          map((entity: Entity) => entity.id),
        ),
        this.$watchAsObservable("selectedAction", { immediate: true }).pipe(
          pluck("newValue"),
        ),
      ).pipe(
        tap(async ([system, workspace, entityId, selectedAction]) => {
          // Or maybe this might? Hard to say.
          if (system && workspace && entityId) {
            let request: IListActionRequest = {
              workspaceId: workspace.id,
              systemId: system.id,
              // @ts-ignore
              entityId,
            };
            if (selectedAction) {
              request.actionName = selectedAction as string;
            }
            let reply = await WorkflowDal.listAction(request);
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              this.workflowRuns = reply.workflowRuns;
            }
          }
        }),
      ),
    };
  },
  methods: {
    showRun(workflowRun: WorkflowRunType): boolean {
      let show = true;
      if (workflowRun.state == WorkflowRunState.Failure) {
        if (this.isFailedVisible == false) {
          show = false;
        }
      } else if (workflowRun.state == WorkflowRunState.Success) {
        if (this.isSucceededVisible == false) {
          show = false;
        }
      }
      return show;
    },
    async runThisAction(): Promise<void> {
      // @ts-ignore
      if (this.system && this.workspace) {
        let reply = await WorkflowDal.runAction({
          // @ts-ignore
          systemId: this.system.id,
          // @ts-ignore
          workspaceId: this.workspace.id,
          entityId: this.entity.id,
          actionName: this.selectedAction,
        });
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        } else {
          refreshActivitySummary$.next(true);
          //this.workflowRuns.unshift({
          //  workflowRun: reply.workflowRun,
          //  steps: [],
          //});
        }
      }
    },
    successesFilterClasses(): Record<string, any> {
      let classes: Record<string, any> = {};

      if (this.isSucceededVisible) {
        classes["text-green-400"] = true;
      } else {
        classes["text-gray-500"] = true;
      }
      return classes;
    },
    failuresFilterClasses(): Record<string, any> {
      let classes: Record<string, any> = {};

      if (this.isFailedVisible) {
        classes["text-red-400"] = true;
      } else {
        classes["text-gray-500"] = true;
      }
      return classes;
    },
    toggleShowSucceeded() {
      this.isSucceededVisible = !this.isSucceededVisible;
    },
    toggleShowFailed() {
      this.isFailedVisible = !this.isFailedVisible;
    },
  },
});
</script>

<style scoped>
.workflow-runs {
  background-color: #212324;
}

.property-section-bg-color {
  background-color: #292c2d;
}

.ok {
  color: #86f0ad;
}

.warning {
  color: #f0d286;
}

.error {
  color: #f08686;
}

.unknown {
  color: #5b6163;
}
</style>
