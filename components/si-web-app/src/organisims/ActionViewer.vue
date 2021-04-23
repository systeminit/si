<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center pt-2 pb-2 pl-6 pr-6 text-base text-white property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} actions
      </div>
    </div>
    <div
      class="relative flex flex-row items-center pt-2 pb-2 pl-6 pr-6 text-base text-white bg-black"
    >
      <div class="flex">
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
        :disabled="!selectedAction || editMode"
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

    <div
      v-if="schema && schema.qualifications"
      class="flex w-full pt-2 overflow-auto"
    >
      <div class="flex flex-col w-full">
        <VueJsonPretty :data="workflowRuns" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>

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
import { PlayCircleIcon } from "vue-feather-icons";
import VueJsonPretty from "vue-json-pretty";
import { ILabelListItem } from "@/api/sdf/dal";
import {
  system$,
  workspace$,
  editMode$,
  workflowRuns$,
  workflowRunSteps$,
  workflowRunStepEntities$,
} from "@/observables";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { combineLatest } from "rxjs";
import { tap, pluck, map } from "rxjs/operators";

interface Data {
  dryRun: boolean;
  selectedAction: string;
  workflowRuns: IListActionReplySuccess["workflowRuns"];
}

// The critical data are the 'worfklowRun', 'workflowRunStep', and 'workflowRunStepEntity'.

export default Vue.extend({
  name: "ActionViewer",
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
  },
  components: {
    PlayCircleIcon,
    SiSelect,
    VueJsonPretty,
  },
  data(): Data {
    return {
      dryRun: false,
      selectedAction: "",
      workflowRuns: [],
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
  },
  subscriptions: function(this: any): Record<string, any> {
    return {
      editMode: editMode$,
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
          //this.workflowRuns.unshift({
          //  workflowRun: reply.workflowRun,
          //  steps: [],
          //});
        }
      }
    },
  },
});
</script>
