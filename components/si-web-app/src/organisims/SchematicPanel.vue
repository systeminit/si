<template>
  <Panel
    initialPanelType="schematic"
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
      <div class="flex flex-row">
        <div class="min-w-max">
          <SiSelect
            size="xs"
            id="schematicSelect"
            name="schematicSelect"
            :options="schematicKinds"
            v-model="schematicKind"
            class="pl-1"
            :styling="schematicSelectorStyling"
          />
        </div>
        <!-- This is irrelevant for now; eventually, it should set the system -->

        <div class="min-w-max">
          <SiSelect
            size="xs"
            id="systemSelect"
            name="systemSelect"
            :options="systemsList"
            class="pl-1"
            :styling="schematicSelectorStyling"
            v-if="schematicKind === 'deployment'"
          />
        </div>

        <!-- 
      <div class="flex flex-row" v-if="schematicKind === 'component'">
        <SiSelect
          size="xs"
          id="schematicPanelNodePin"
          name="schematicPanelNodePin"
          :options="nodeList()"
          v-if="nodeList()"
          v-model="pinnedNodeId"
          class="pl-1"
          :disabled="selectionIsTracked"
        />
        <button class="pl-1 focus:outline-none" @click="toggleSelectionTrack()">
          <TargetIcon size="0.9x" :class="targetIconStyling()" />
        </button>
      </div>
      -->

        <NodeAddMenu
          class="pl-4"
          :filter="addMenuFilters"
          @selected="nodeCreate"
          :disabled="!addMenuEnabled"
        />
      </div>
    </template>
    <template v-slot:content>
      <div class="relative w-full h-full">
        <SchematicViewer
          class="absolute z-10"
          :schematic="schematic"
          :schematicKind="schematicKind"
          :positionCtx="positionCtx"
          :rootObjectId="rootObjectId"
          ref="graphViewer"
        />
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";
import _ from "lodash";

import {
  SchematicKind,
  Schematic,
  ISchematicNode,
} from "@/api/sdf/model/schematic";
import { ILabelList } from "@/api/sdf/dal";

import Panel from "@/molecules/Panel.vue";
import NodeAddMenu from "@/molecules/NodeAddMenu.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import SchematicViewer from "@/organisims/SchematicViewer.vue";

import {
  system$,
  editMode$,
  workspace$,
  applicationId$,
  deploymentSchematicSelectNode$,
  changeSet$,
  editSession$,
  nodePositionUpdated$,
  schematicUpdated$,
  schematicSelectNode$,
  edgeDeleted$,
  schematicPanelKind$,
  restoreSchematicPanelKind$,
  nameAttributeChanged$,
  workflowRuns$,
  resources$,
  entityQualifications$,
} from "@/observables";
import { combineLatest, of, BehaviorSubject } from "rxjs";
import { switchMap, pluck, tap } from "rxjs/operators";
import {
  IGetApplicationSystemSchematicRequest,
  getApplicationSystemSchematic,
  IGetSchematicReply,
  SchematicDal,
  INodeCreateForApplicationRequest,
} from "@/api/sdf/dal/schematicDal";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { EntityMenuFilters } from "si-registry";

interface Data {
  schematicKind: SchematicKind;
  schematic: Schematic | null;
  schematicRootObjectId: string | null;
}

export default Vue.extend({
  name: "SchematicPanel",
  props: {
    panelIndex: Number,
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
    isVisible: Boolean,
    isMaximizedContainerEnabled: Boolean,
  },
  components: {
    Panel,
    SiSelect,
    SchematicViewer,
    NodeAddMenu,
  },
  data(): Data {
    return {
      schematicKind: SchematicKind.Deployment,
      schematic: null,
      schematicRootObjectId: null,
    };
  },
  subscriptions: function(this: any): Record<string, any> {
    let selectedSchematicKind$ = this.$watchAsObservable("schematicKind", {
      immediate: true,
    }).pipe(pluck("newValue"));

    // and elizabeth loves you
    let positionCtx$ = combineLatest(
      selectedSchematicKind$,
      deploymentSchematicSelectNode$,
      applicationId$,
    ).pipe(
      switchMap(
        ([
          selectedSchematicKind,
          deploymentSelectedSchematicNode,
          applicationId,
        ]) => {
          if (
            deploymentSelectedSchematicNode &&
            selectedSchematicKind == SchematicKind.Component
          ) {
            return of(
              `${deploymentSelectedSchematicNode.object.id}.${selectedSchematicKind}`,
            );
          } else {
            return of(`${applicationId}.${selectedSchematicKind}`);
          }
        },
      ),
    );

    let rootObjectId$ = combineLatest(
      selectedSchematicKind$,
      applicationId$,
      deploymentSchematicSelectNode$,
    ).pipe(
      switchMap(
        ([schematicKind, applicationId, deploymentSchematicSelectNode]) => {
          if (schematicKind == SchematicKind.Deployment) {
            if (applicationId) {
              return of(applicationId);
            } else {
              return of("noSelectedApplicationNode");
            }
          } else {
            if (deploymentSchematicSelectNode) {
              return of(deploymentSchematicSelectNode.object.id);
            } else {
              return of("noSelectedDeploymentNode");
            }
          }
        },
      ),
    );

    const refreshSchematic$ = new BehaviorSubject<boolean>(true);

    let schematicUpdateCallback$ = schematicUpdated$.pipe(
      tap(payload => {
        if (payload.schematicKind == this.schematicKind) {
          this.schematic = payload.schematic;
        }
      }),
    );

    let loadSchematic$ = combineLatest(
      workspace$,
      system$,
      selectedSchematicKind$,
      rootObjectId$,
      changeSet$,
      nodePositionUpdated$,
      edgeDeleted$,
      refreshSchematic$,
    ).pipe(
      switchMap(
        ([
          workspace,
          system,
          selectedSchematicKind,
          rootObjectId,
          changeSet,
          _nodePositionUpdated,
          _edgeDeleted,
          _refreshSchematic,
        ]) => {
          if (
            rootObjectId == "noSelectedDeploymentNode" ||
            rootObjectId == "noSelectedApplicationNode"
          ) {
            this.schematic = null;
            return of({
              error: { message: "no selected deployment node", code: 42 },
            });
          }

          if (workspace && system && selectedSchematicKind && rootObjectId) {
            let includeRootNode = false;
            if (selectedSchematicKind == SchematicKind.Component) {
              includeRootNode = true;
            }
            let request: IGetApplicationSystemSchematicRequest = {
              workspaceId: workspace.id,
              rootObjectId: rootObjectId,
              systemId: system.id,
              includeRootNode,
              schematicKind: selectedSchematicKind,
            };
            if (changeSet) {
              request["changeSetId"] = changeSet.id;
            }
            if (this.editSession) {
              request["editSessionId"] = this.editSession.id;
            }
            this.schematicRootObjectId = rootObjectId;
            return getApplicationSystemSchematic(request);
          } else {
            return of({ error: { message: "cannot get schema", code: 42 } });
          }
        },
      ),
      tap((reply: IGetSchematicReply) => {
        if (reply.error) {
          if (reply.error.code == 406) {
            if (this.schematicKind == SchematicKind.Component) {
              deploymentSchematicSelectNode$.next(null);
              schematicSelectNode$.next(null);
            } else {
              schematicSelectNode$.next(null);
            }
          } else if (reply.error.code != 42) {
            emitEditorErrorMessage(reply.error.message);
          }
        } else {
          this.schematic = reply.schematic;
        }
      }),
    );
    return {
      editMode: editMode$,
      selectedSchematicKind: selectedSchematicKind$,
      system: system$,
      deploymentSchematicSelectNode: deploymentSchematicSelectNode$,
      applicationId: applicationId$,
      rootObjectId: rootObjectId$,
      loadSchematic: loadSchematic$,
      positionCtx: positionCtx$,
      changeSet: changeSet$,
      editSession: editSession$,
      workspace: workspace$,
      schematicUpdateCallback: schematicUpdateCallback$,
      edgeDeleted: edgeDeleted$,
      saveSchematicPanelState: selectedSchematicKind$.pipe(
        tap(schematicKind => {
          let applicationId = this.$route.params["applicationId"];
          schematicPanelKind$.next({
            panelRef: this.panelRef,
            // @ts-ignore
            schematicKind,
            applicationId,
          });
        }),
      ),
      restoreSchematicPanelState: restoreSchematicPanelKind$.pipe(
        tap(schematicState => {
          let applicationId = this.$route.params["applicationId"];
          if (
            schematicState.panelRef == this.panelRef &&
            schematicState.applicationId == applicationId
          ) {
            this.schematicKind = schematicState.schematicKind;
          }
        }),
      ),
      nameAttributeChanged: nameAttributeChanged$.pipe(
        tap(payload => {
          if (
            payload &&
            this.schematic &&
            this.schematic.nodes[payload.nodeId]
          ) {
            this.schematic.nodes[payload.nodeId].object.name = payload.newValue;
            refreshSchematic$.next(true);
          }
        }),
      ),
      workflowRunUpdate: workflowRuns$.pipe(
        tap(workflowRun => {
          if (
            workflowRun.ctx.entity &&
            workflowRun.ctx.system &&
            this.schematic
          ) {
            const nodeId = workflowRun.ctx.entity.nodeId;
            const systemId = workflowRun.ctx.system.id;
            if (this.schematic.nodes[nodeId]) {
              if (this.schematic.nodes[nodeId].workflowRuns[systemId]) {
                Vue.set(
                  this.schematic.nodes[nodeId].workflowRuns[systemId],
                  "workflowRun",
                  workflowRun,
                );
              } else {
                Vue.set(this.schematic.nodes[nodeId].workflowRuns, systemId, {
                  workflowRun,
                });
              }
            }
          }
        }),
      ),
      resourceUpdate: resources$.pipe(
        tap(resource => {
          if (this.schematic) {
            for (let node of Object.values(this.schematic.nodes)) {
              // @ts-ignore
              if (node.node.objectId == resource.entityId) {
                Vue.set(
                  // @ts-ignore
                  this.schematic.nodes[node.node.id].resources,
                  resource.systemId,
                  resource,
                );
              }
            }
          }
        }),
      ),
      qualificationsUpdate: entityQualifications$.pipe(
        tap(qualification => {
          if (this.schematic) {
            for (let node of Object.values(this.schematic.nodes)) {
              // @ts-ignore
              if (node.node.objectId == qualification.entityId) {
                let updated = false;
                // @ts-ignore
                for (let x = 0; x < node.qualifications.length; x++) {
                  // @ts-ignore
                  let qcheck = node.qualifications[x];
                  if (qcheck.name == qualification.name) {
                    Vue.set(
                      // @ts-ignore
                      this.schematic.nodes[node.node.id].qualifications,
                      x,
                      qualification,
                    );
                    updated = true;
                  }
                }
                if (!updated) {
                  // @ts-ignore
                  this.schematic.nodes[node.node.id].qualifications.push(
                    qualification,
                  );
                }
              }
            }
          }
        }),
      ),
    };
  },
  computed: {
    addMenuEnabled(this: any): boolean {
      if (this.schematicKind == SchematicKind.Component) {
        if (
          this.editMode &&
          !_.isNull(this.deploymentSchematicSelectNode) &&
          this.deploymentSchematicSelectNode != "noSelectedDeploymentNode"
        ) {
          return true;
        } else {
          return false;
        }
      } else {
        return this.editMode;
      }
    },
    addMenuFilters(this: any): EntityMenuFilters {
      if (this.schematicKind == SchematicKind.Deployment) {
        return {
          rootEntityType: "application",
          schematicKind: this.schematicKind,
        };
      } else {
        if (
          this.deploymentSchematicSelectNode &&
          this.deploymentSchematicSelectNode != "noSelectedDeploymentNode"
        ) {
          return {
            rootEntityType: this.deploymentSchematicSelectNode.object
              .entityType,
            schematicKind: this.schematicKind,
          };
        } else {
          return {
            rootEntityType: "never",
            schematicKind: this.schematicKind,
          };
        }
      }
    },
    schematicKinds(): ILabelList {
      let labels: ILabelList = [];
      for (const value in SchematicKind) {
        labels.push({ label: value, value: _.lowerCase(value) });
      }
      return labels;
    },
    systemsList(): ILabelList {
      // @ts-ignore
      if (this.system) {
        // @ts-ignore
        return [{ value: this.system.id, label: this.system.name }];
      } else {
        return [{ value: "", label: "" }];
      }
    },
    schematicSelectorStyling(): Record<string, any> {
      let classes: Record<string, any> = {};
      classes["bg-selectordark"] = true;
      classes["text-gray-400"] = true;
      classes["border-gray-800"] = true;
      return classes;
    },
  },
  methods: {
    async nodeCreate(
      this: any,
      entityType: string,
      event: MouseEvent,
    ): Promise<void> {
      if (
        this.applicationId &&
        this.workspace &&
        this.changeSet &&
        this.editSession
      ) {
        const request: INodeCreateForApplicationRequest = {
          entityType,
          applicationId: this.applicationId,
          workspaceId: this.workspace.id,
          changeSetId: this.changeSet.id,
          editSessionId: this.editSession.id,
          schematicKind: this.schematicKind,
        };
        if (this.schematicKind == SchematicKind.Component) {
          const deploymentSelectedEntityId = this.deploymentSchematicSelectNode
            .object.id;
          request["deploymentSelectedEntityId"] = deploymentSelectedEntityId;
        }

        let reply = await SchematicDal.nodeCreateForApplication(request);

        if (!reply.error) {
          if (reply.schematic) {
            this.schematic = reply.schematic;
            schematicUpdated$.next({
              schematicKind: this.schematicKind,
              schematic: reply.schematic,
            });
          }
          schematicSelectNode$.next(reply.node);
          if (this.schematicKind == SchematicKind.Deployment) {
            deploymentSchematicSelectNode$.next(reply.node);
          }

          // @ts-ignore
          this.$refs.graphViewer.onNodeCreate(reply.node.node.id, event);
        } else {
          emitEditorErrorMessage(reply.error.message);
        }
      }
    },
    onInitialMaximizedFullUpdates(_value: boolean) {
      // TODO: This should be refactored, because it's overly coupled.
      // @ts-ignore
      this.$refs.graphViewer.updateCanvasPosition();
    },
  },
  watch: {
    initialMaximizedFull(value) {
      this.onInitialMaximizedFullUpdates(value);
    },
  },
});
</script>
