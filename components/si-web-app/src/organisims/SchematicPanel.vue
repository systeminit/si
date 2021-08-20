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
            :disabled="isPinned"
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

        <button
          class="pl-4 text-sm focus:outline-none"
          @click="togglePinned()"
          v-show="schematicKind == 'component'"
        >
          <LockIcon size="1.0x" class="locked" v-if="isPinned" />
          <UnlockIcon size="1.0x" class="unlocked" v-else />
        </button>

        <NodeAddMenu
          class="pl-4"
          :filter="addMenuFilters"
          @selected="nodeCreate"
          :disabled="!addMenuEnabled"
        />

        <NodeLinkMenu
          class="pl-4"
          :positionCtx="positionCtx"
          :filter="addMenuFilters"
          @selected="nodeLink"
          :disabled="!linkMenuEnabled"
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
  schematicKindfromString,
} from "@/api/sdf/model/schematic";
import { ILabelList } from "@/api/sdf/dal";

import Panel from "@/molecules/Panel.vue";
import NodeAddMenu from "@/molecules/NodeAddMenu.vue";
import NodeLinkMenu from "@/molecules/NodeLinkMenu.vue";
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
  nodeDeleted$,
  refreshSchematic$,
} from "@/observables";
import { combineLatest, of, BehaviorSubject, Observable } from "rxjs";
import { switchMap, pluck, tap, filter, skipWhile, map } from "rxjs/operators";
import {
  IGetApplicationSystemSchematicRequest,
  getApplicationSystemSchematic,
  IGetSchematicReply,
  SchematicDal,
  INodeCreateForApplicationRequest,
  INodeLinkForApplicationRequest,
} from "@/api/sdf/dal/schematicDal";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { EntityMenuFilters, LinkNodeItem } from "si-registry";
import { LockIcon, UnlockIcon } from "vue-feather-icons";
import { IWorkspace } from "@/api/sdf/model/workspace";
import { IEntity } from "@/api/sdf/model/entity";
import { IChangeSet } from "@/api/sdf/model/changeSet";
import { IEditSession } from "@/api/sdf/model/editSession";

interface Data {
  schematicKind: SchematicKind;
  schematic: Schematic | null;
  rootObjectId: string | null;
  isPinned: boolean;
  restoring: boolean;
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
    LockIcon,
    NodeAddMenu,
    NodeLinkMenu,
    Panel,
    SchematicViewer,
    SiSelect,
    UnlockIcon,
  },
  data(): Data {
    return {
      schematicKind: SchematicKind.Deployment,
      schematic: null,
      rootObjectId: null,
      isPinned: false,
      restoring: true,
    };
  },
  subscriptions() {
    const selectedSchematicKind$: Observable<Data["schematicKind"]> = this.$watchAsObservable<
      Data["schematicKind"]
    >(
      () =>
        // @ts-ignore
        this.schematicKind,
      {
        immediate: true,
      },
    ).pipe(pluck("newValue"));

    const isPinned$ = this.$watchAsObservable<Data["isPinned"]>(
      () =>
        // @ts-ignore
        this.isPinned,
      {
        immediate: true,
      },
    ).pipe(pluck("newValue"));

    const restoring$ = this.$watchAsObservable<boolean>(
      () =>
        // @ts-ignore
        this.restoring,
      {
        immediate: true,
      },
    ).pipe(pluck("newValue"));

    const rootObjectId$ = this.$watchAsObservable<Data["rootObjectId"]>(
      () =>
        // @ts-ignore
        this.rootObjectId,
      {
        immediate: true,
      },
    ).pipe(pluck("newValue"));

    // and elizabeth loves you
    const positionCtx$ = combineLatest([
      selectedSchematicKind$,
      applicationId$,
      deploymentSchematicSelectNode$,
      rootObjectId$,
      restoring$,
    ]).pipe(
      map(
        ([
          selectedSchematicKind,
          applicationId,
          deploymentSchematicSelectNode,
          rootObjectId,
          restoring,
        ]) => {
          return {
            selectedSchematicKind,
            applicationId,
            deploymentSchematicSelectNode,
            rootObjectId,
            restoring,
          };
        },
      ),
      skipWhile(({ restoring }) => restoring),
      switchMap(({ selectedSchematicKind, applicationId, rootObjectId }) => {
        switch (selectedSchematicKind) {
          case SchematicKind.Deployment:
            return of(`${applicationId}.${selectedSchematicKind}`);
          case SchematicKind.Component:
            if (rootObjectId) {
              return of(`${rootObjectId}.${selectedSchematicKind}`);
            } else {
              return of(`${applicationId}.${selectedSchematicKind}`);
            }
          default:
            throw Error(
              `Unknown SchematicKind member: ${selectedSchematicKind}`,
            );
        }
      }),
    );

    const setRootObjectId$ = combineLatest(
      selectedSchematicKind$,
      applicationId$,
      deploymentSchematicSelectNode$,
      isPinned$,
      restoring$,
    ).pipe(
      map(
        ([
          selectedSchematicKind,
          applicationId,
          deploymentSchematicSelectNode,
          isPinned,
          restoring,
        ]) => {
          return {
            selectedSchematicKind,
            applicationId,
            deploymentSchematicSelectNode,
            isPinned,
            restoring,
          };
        },
      ),
      skipWhile(({ restoring }) => restoring),
      filter(({ isPinned }) => !isPinned),
      tap(
        ({
          selectedSchematicKind,
          applicationId,
          deploymentSchematicSelectNode,
        }) => {
          switch (selectedSchematicKind) {
            case SchematicKind.Deployment:
              // @ts-ignore
              this.rootObjectId = applicationId;
              break;
            case SchematicKind.Component:
              if (deploymentSchematicSelectNode) {
                // @ts-ignore
                this.rootObjectId = deploymentSchematicSelectNode.object.id;
              } else {
                // @ts-ignore
                this.rootObjectId = null;
              }
              break;
            default:
              throw Error(
                `Unknown SchematicKind member: ${selectedSchematicKind}`,
              );
          }
        },
      ),
    );

    const internalRefreshSchematic$ = new BehaviorSubject<boolean>(true);

    const schematicUpdateCallback$ = combineLatest(
      schematicUpdated$,
      restoring$,
    ).pipe(
      map(([schematicUpdated, restoring]) => {
        return { schematicUpdated, restoring };
      }),
      skipWhile(({ restoring }) => restoring),
      tap(({ schematicUpdated }) => {
        if (
          // @ts-ignore
          schematicUpdated.schematicKind == this.schematicKind &&
          // @ts-ignore
          schematicUpdated.rootObjectId == this.rootObjectId
        ) {
          // @ts-ignore
          this.schematic = schematicUpdated.schematic;
        }
      }),
    );

    const loadSchematic$ = combineLatest([
      workspace$,
      system$,
      selectedSchematicKind$,
      changeSet$,
      editSession$,
      rootObjectId$,
      restoring$,
      // the remaining observables are strictly for triggering reloads
      nodePositionUpdated$,
      edgeDeleted$,
      internalRefreshSchematic$,
      refreshSchematic$,
    ]).pipe(
      // FUCKKKK!
      // `combineLatest` only type hints up to 6 params or array entries before
      // turning all back to `any[]`, so we're going to map this mess into
      // a hash that's typed AGAIN so down the chain gets proper types to
      // eliminate logic errors. And yes, you'll make logic errors, Fletcher
      // guarantees it.
      //
      // See: https://stackoverflow.com/q/56250218
      map(
        ([
          workspace,
          system,
          selectedSchematicKind,
          changeSet,
          editSession,
          rootObjectId,
          restoring,
          _nodePositionUpated,
          _edgeDeleted,
          _internalRefreshSchematic,
          _refreshSchematic,
        ]) => {
          return {
            workspace,
            system,
            selectedSchematicKind,
            changeSet,
            editSession,
            rootObjectId,
            restoring,
          } as {
            workspace: IWorkspace | null;
            system: IEntity | null;
            selectedSchematicKind: SchematicKind;
            changeSet: IChangeSet | null;
            editSession: IEditSession | null;
            rootObjectId: string | null;
            restoring: boolean;
          };
        },
      ),
      skipWhile(({ restoring }) => restoring),
      switchMap(
        ({
          workspace,
          system,
          selectedSchematicKind,
          changeSet,
          editSession,
          rootObjectId,
        }) => {
          if (!rootObjectId) {
            // @ts-ignore
            this.schematic = null;
            return of({
              error: { message: "no selected deployment node", code: 42 },
            });
          }

          if (workspace && system && selectedSchematicKind) {
            let includeRootNode = false;
            if (selectedSchematicKind == SchematicKind.Component) {
              includeRootNode = true;
            }
            let request: IGetApplicationSystemSchematicRequest = {
              workspaceId: workspace.id,
              rootObjectId,
              systemId: system.id,
              includeRootNode,
              schematicKind: selectedSchematicKind,
            };
            if (changeSet) {
              request["changeSetId"] = changeSet.id;
            }
            if (editSession) {
              request["editSessionId"] = editSession.id;
            }
            return getApplicationSystemSchematic(request);
          } else {
            return of({ error: { message: "cannot get schema", code: 42 } });
          }
        },
      ),
      tap((reply: IGetSchematicReply) => {
        if (reply.error) {
          if (reply.error.code == 406) {
            // @ts-ignore
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
          // @ts-ignore
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
      setRootObjectId: setRootObjectId$,
      loadSchematic: loadSchematic$,
      positionCtx: positionCtx$,
      changeSet: changeSet$,
      editSession: editSession$,
      workspace: workspace$,
      schematicUpdateCallback: schematicUpdateCallback$,
      edgeDeleted: edgeDeleted$,
      saveSchematicPanelState: combineLatest(
        selectedSchematicKind$,
        applicationId$,
        rootObjectId$,
        isPinned$,
        restoring$,
      ).pipe(
        map(
          ([
            schematicKind,
            applicationId,
            rootObjectId,
            isPinned,
            restoring,
          ]) => {
            return {
              schematicKind,
              applicationId,
              rootObjectId,
              isPinned,
              restoring,
            };
          },
        ),
        skipWhile(({ restoring }) => restoring),
        tap(({ schematicKind, applicationId, rootObjectId, isPinned }) => {
          if (schematicKind && applicationId) {
            // @ts-ignore
            const panelRef = this.panelRef;

            schematicPanelKind$.next({
              panelRef,
              applicationId,
              schematicKind,
              rootObjectId,
              isPinned,
            });
          }
        }),
      ),
      nameAttributeChanged: nameAttributeChanged$.pipe(
        tap(payload => {
          // @ts-ignore
          const schematic = this.schematic as Data["schematic"];

          if (payload && schematic && schematic.nodes[payload.nodeId]) {
            schematic.nodes[payload.nodeId].object.name = payload.newValue;
            internalRefreshSchematic$.next(true);
          }
        }),
      ),
      nodeDeleted: nodeDeleted$.pipe(
        tap(payload => {
          // @ts-ignore
          const schematic = this.schematic as Data["schematic"];

          if (payload && schematic && schematic.nodes[payload.nodeId]) {
            internalRefreshSchematic$.next(true);
          }
        }),
      ),
      workflowRunUpdate: workflowRuns$.pipe(
        tap(workflowRun => {
          // @ts-ignore
          const schematic = this.schematic as Data["schematic"];

          if (workflowRun.ctx.entity && workflowRun.ctx.system && schematic) {
            const nodeId = workflowRun.ctx.entity.nodeId;
            const systemId = workflowRun.ctx.system.id;
            if (schematic.nodes[nodeId]) {
              if (schematic.nodes[nodeId].workflowRuns[systemId]) {
                Vue.set(
                  schematic.nodes[nodeId].workflowRuns[systemId],
                  "workflowRun",
                  workflowRun,
                );
              } else {
                Vue.set(schematic.nodes[nodeId].workflowRuns, systemId, {
                  workflowRun,
                });
              }
            }
          }
        }),
      ),
      resourceUpdate: resources$.pipe(
        tap(resource => {
          // @ts-ignore
          const schematic = this.schematic as Data["schematic"];

          if (schematic) {
            for (const node of Object.values(schematic.nodes)) {
              if (node.object.id == resource.entityId) {
                Vue.set(
                  schematic.nodes[node.node.id].resources,
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
          // @ts-ignore
          const schematic = this.schematic as Data["schematic"];

          if (schematic) {
            for (let node of Object.values(schematic.nodes)) {
              if (node.object.id == qualification.entityId) {
                let updated = false;
                for (let x = 0; x < node.qualifications.length; x++) {
                  let qcheck = node.qualifications[x];
                  if (qcheck.name == qualification.name) {
                    Vue.set(
                      schematic.nodes[node.node.id].qualifications,
                      x,
                      qualification,
                    );
                    updated = true;
                  }
                }
                if (!updated) {
                  schematic.nodes[node.node.id].qualifications.push(
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
    linkMenuEnabled(this: any): boolean {
      if (this.schematicKind == SchematicKind.Component) {
        if (this.editMode && !_.isNull(this.deploymentSchematicSelectNode)) {
          return true;
        } else {
          return false;
        }
      } else {
        return this.editMode;
      }
    },
    addMenuEnabled(this: any): boolean {
      if (this.schematicKind == SchematicKind.Component) {
        if (this.editMode && !_.isNull(this.deploymentSchematicSelectNode)) {
          return true;
        } else {
          return false;
        }
      } else {
        return this.editMode;
      }
    },
    addMenuFilters(): EntityMenuFilters {
      switch (this.schematicKind) {
        case SchematicKind.Deployment:
          return {
            rootEntityType: "application",
            schematicKind: this.schematicKind,
          };
        case SchematicKind.Component:
          let rootEntityType = "never";

          if (this.rootObjectId && this.schematic) {
            const ret = _.find(
              Object.values(this.schematic.nodes),
              n => n.object.id == this.rootObjectId,
            );
            if (ret) {
              rootEntityType = ret.node.objectType;
            }
          }

          return {
            rootEntityType,
            schematicKind: this.schematicKind,
          };
        default:
          throw Error(`Unknown SchematicKind member: ${this.schematicKind}`);
      }
    },
    schematicKinds(): ILabelList {
      let labels: ILabelList = [];
      for (const value in SchematicKind) {
        labels.push({ label: value, value: _.lowerCase(value) });
      }
      return labels;
    },
    systemsList(this: any): ILabelList {
      if (this.system) {
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
        if (
          this.schematicKind == SchematicKind.Component &&
          this.rootObjectId &&
          this.schematic
        ) {
          const ret = _.find(
            Object.values(this.schematic.nodes as ISchematicNode[]),
            n => n.object.id == this.rootObjectId,
          );
          if (ret) {
            request["deploymentSelectedEntityId"] = ret.object.id;
          }
        }

        let reply = await SchematicDal.nodeCreateForApplication(request);

        if (!reply.error) {
          if (reply.schematic) {
            this.schematic = reply.schematic;
            schematicUpdated$.next({
              schematicKind: this.schematicKind,
              schematic: reply.schematic,
              rootObjectId: this.rootObjectId,
            });
          }
          schematicSelectNode$.next(reply.node);
          if (this.schematicKind == SchematicKind.Deployment) {
            deploymentSchematicSelectNode$.next(reply.node);
          }

          this.$refs.graphViewer.onNodeCreate(reply.node.node.id, event);
        } else {
          emitEditorErrorMessage(reply.error.message);
        }
      }
    },
    async nodeLink(
      this: any,
      toLink: LinkNodeItem,
      event: MouseEvent,
    ): Promise<void> {
      if (
        this.applicationId &&
        this.workspace &&
        this.changeSet &&
        this.editSession
      ) {
        const request: INodeLinkForApplicationRequest = {
          entityType: toLink.entityType,
          nodeId: toLink.nodeId,
          entityId: toLink.entityId,
          applicationId: this.applicationId,
          workspaceId: this.workspace.id,
          changeSetId: this.changeSet.id,
          editSessionId: this.editSession.id,
          schematicKind: this.schematicKind,
        };
        if (
          this.schematicKind == SchematicKind.Component &&
          this.rootObjectId &&
          this.schematic
        ) {
          const ret = _.find(
            Object.values(this.schematic.nodes as ISchematicNode[]),
            n => n.object.id == this.rootObjectId,
          );
          if (ret) {
            request["deploymentSelectedEntityId"] = ret.object.id;
          }
        }

        let reply = await SchematicDal.nodeLinkForApplication(request);

        if (!reply.error) {
          if (reply.schematic) {
            this.schematic = reply.schematic;
            schematicUpdated$.next({
              schematicKind: this.schematicKind,
              schematic: reply.schematic,
              rootObjectId: this.rootObjectId,
            });
          }
          schematicSelectNode$.next(reply.node);
          if (this.schematicKind == SchematicKind.Deployment) {
            deploymentSchematicSelectNode$.next(reply.node);
          }

          this.$refs.graphViewer.onNodeCreate(reply.node.node.id, event);
        } else {
          emitEditorErrorMessage(reply.error.message);
        }
      }
    },

    onInitialMaximizedFullUpdates(this: any, _value: boolean) {
      // TODO: This should be refactored, because it's overly coupled.
      this.$refs.graphViewer.updateCanvasPosition();
    },
    togglePinned() {
      if (this.isPinned) {
        this.isPinned = false;
      } else {
        this.isPinned = true;
      }
    },
  },
  watch: {
    initialMaximizedFull(value) {
      this.onInitialMaximizedFullUpdates(value);
    },
  },
  created() {
    const ref = this;
    const applicationId = ref.$route.params["applicationId"];

    restoreSchematicPanelKind$.subscribe({
      next(schematicState) {
        if (
          schematicState.panelRef == ref.panelRef &&
          schematicState.applicationId == applicationId
        ) {
          ref.isPinned = schematicState.isPinned;
          ref.schematicKind = schematicKindfromString(
            schematicState.schematicKind,
          );
          ref.rootObjectId = schematicState.rootObjectId;
        }
      },
      complete() {
        ref.restoring = false;
        refreshSchematic$.next(true);
      },
    });
  },
});
</script>

<style scoped>
.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
