<template>
  <Panel
    initialPanelType="attribute"
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
      <div class="flex flex-row items-center justify-between flex-grow">
        <div class="flex flex-row">
          <div class="min-w-max">
            <SiSelect
              size="xs"
              id="attributePanelObjectSelect"
              name="attributePanelObjectSelect"
              :options="entityLabelList.entityList"
              v-if="entityLabelList"
              v-model="selectedEntityId"
              class="pl-1"
              :disabled="selectionIsLocked"
            />
          </div>
          <button
            class="pl-1 focus:outline-none"
            @click="toggleSelectionLock()"
          >
            <UnlockIcon size="1.1x" class="unlocked" v-if="selectionIsLocked" />
            <LockIcon size="1.1x" class="locked" v-else />
          </button>
        </div>

        <div class="flex flex-row items-center">
          <button
            class="pl-1 focus:outline-none"
            :class="attributeViewClasses()"
            @click="switchToAttributeView()"
          >
            <DiscIcon size="1.1x" />
          </button>
          <button
            class="pl-1 focus:outline-none"
            :class="codeViewClasses()"
            @click="switchToCodeView()"
          >
            <CodeIcon size="1.1x" />
          </button>

          <button
            class="pl-1 focus:outline-none"
            :class="qualificationViewClasses()"
            @click="switchToQualificationView()"
          >
            <CheckSquareIcon size="1.1x" />
          </button>
        </div>

        <div class="flex flex-row items-center">
          <!-- <button
          class="pl-1 text-white focus:outline-none"
          :class="eventViewClasses()"
          @click="switchToEventView()"
        >
          <RadioIcon size="1.1x" />
        </button> -->

          <button
            class="pl-1 text-white focus:outline-none"
            :class="connectionViewClasses()"
            @click="switchTocCnnectionView()"
          >
            <Link2Icon size="1.1x" />
          </button>
        </div>

        <div class="flex items-center">
          <button
            class="pl-1 text-white focus:outline-none"
            :class="actionViewClasses()"
            @click="switchToActionView()"
          >
            <PlayIcon size="1.1x" />
          </button>

          <button
            class="pl-1 text-white focus:outline-none"
            :class="resourceViewClasses()"
            @click="switchToResourceView()"
          >
            <BoxIcon size="1.1x" />
          </button>
        </div>

        <!--
      <button
        class="pl-1 text-white focus:outline-none"
        :class="refreshClasses"
        @click="refreshObject()"
        :disabled="!needsRefresh"
      >
        <RefreshCwIcon size="1.1x" />
      </button>
        -->
      </div>
    </template>
    <template v-slot:content>
      <div class="flex flex-row w-full h-full" v-if="entity">
        <AttributeViewer
          v-if="activeView == 'attribute'"
          :entity="entity"
          :diff="diff"
          :qualifications="qualifications"
          :starting="qualificationStart"
          :resource="resource"
        />
        <CodeViewer
          v-else-if="activeView == 'code'"
          :entity="entity"
          :diff="diff"
        />
        <QualificationViewer
          v-else-if="activeView == 'qualification'"
          :entity="entity"
          :qualifications="qualifications"
          :starting="qualificationStart"
        />
        <ActionViewer
          v-else-if="activeView == 'action'"
          :entity="entity"
          :resource="resource"
        />
        <ResourceViewer
          v-else-if="activeView == 'resource'"
          :entity="entity"
          :resource="resource"
        />
        <ConnectionViewer
          v-else-if="activeView == 'connection'"
          :entity="entity"
          :connections="connections"
        />

        <div v-else class="text-xs">
          Not implemented
          <div>
            <VueJsonPretty :data="entity" />
          </div>
        </div>

        <!--
        <CodeViewer
          v-else-if="activeView == 'code'"
          :attributePanelStoreCtx="attributePanelStoreCtx"
        />
        -->
      </div>
      <div class="flex w-full" v-else>
        <div
          class="flex flex-col items-center justify-center w-full h-full align-middle"
        >
          <img
            width="300px"
            :src="require('@/assets/images/cheech-and-chong.svg')"
          />
        </div>
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";

import Panel from "@/molecules/Panel.vue";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import SiSelect from "@/atoms/SiSelect.vue";
import {
  UnlockIcon,
  LockIcon,
  CodeIcon,
  // RadioIcon,
  DiscIcon,
  CheckSquareIcon,
  PlayIcon,
  Link2Icon,
  BoxIcon,
} from "vue-feather-icons";
import "vue-json-pretty/lib/styles.css";
import { Entity } from "@/api/sdf/model/entity";
import { Connections, IGetConnectionsReply } from "@/api/sdf/dal/attributeDal";
import AttributeViewer from "@/organisims/AttributeViewer.vue";
import QualificationViewer from "@/organisims/QualificationViewer.vue";
import ActionViewer from "@/organisims/ActionViewer.vue";
import CodeViewer from "@/organisims/CodeViewer.vue";
import ConnectionViewer from "@/organisims/ConnectionViewer.vue";
import ResourceViewer from "@/organisims/ResourceViewer.vue";
import {
  loadEntityForEdit,
  loadConnections,
  attributePanelEntityUpdates$,
  entityLabelList$,
  schematicSelectNode$,
  entityQualifications$,
  entityQualificationStart$,
  changeSet$,
  editSession$,
  refreshEntityLabelList$,
  workspace$,
  system$,
  resources$,
  attributePanelState$,
  restoreAttributePanelState$,
} from "@/observables";
import { combineLatest } from "rxjs";
import { pluck, switchMap, tap } from "rxjs/operators";
import { Diff } from "@/api/sdf/model/diff";
import { ISchematicNode } from "@/api/sdf/model/schematic";
import {
  Qualification,
  QualificationStart,
} from "@/api/sdf/model/qualification";
import VueJsonPretty from "vue-json-pretty";
import _ from "lodash";
import { Resource } from "si-entity";
import { IGetResourceRequest, ResourceDal } from "@/api/sdf/dal/resourceDal";
import { IGetEntityReply } from "@/api/sdf/dal/attributeDal";

interface IData {
  isLoading: boolean;
  selectedEntityId: string;
  selectedNode: ISchematicNode | null;
  selectionIsLocked: boolean;
  activeView:
    | "action"
    | "attribute"
    | "code"
    | "connection"
    | "event"
    | "qualification"
    | "resource";
  entity: Entity | null;
  connections: Connections;
  resource: Resource | null;
  diff: Diff;
  qualifications: Qualification[];
  qualificationStart: QualificationStart[];
}

export default Vue.extend({
  name: "AttributePanel",
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
    DiscIcon,
    BoxIcon,
    // RadioIcon,
    CodeIcon,
    LockIcon,
    UnlockIcon,
    CheckSquareIcon,
    AttributeViewer,
    QualificationViewer,
    CodeViewer,
    PlayIcon,
    ActionViewer,
    ConnectionViewer,
    Link2Icon,
    VueJsonPretty,
    ResourceViewer,
  },
  data(): IData {
    return {
      isLoading: false,
      selectedEntityId: "",
      selectedNode: null,
      selectionIsLocked: true,
      activeView: "attribute",
      entity: null,
      connections: {
        inbound: [],
        outbound: [],
      },
      resource: null,
      diff: [],
      qualifications: [],
      qualificationStart: [],
    };
  },
  subscriptions(this: any): Record<string, any> {
    const selectedEntityId$ = this.$watchAsObservable("selectedEntityId", {
      immediate: true,
    }).pipe(pluck("newValue"));

    const selectionIsLocked$ = this.$watchAsObservable("selectionIsLocked", {
      immediate: true,
    }).pipe(pluck("newValue"));

    const activeView$ = this.$watchAsObservable("activeView", {
      immediate: true,
    }).pipe(pluck("newValue"));

    const entityForEdit$ = selectedEntityId$.pipe(
      switchMap((entityId: string) => loadEntityForEdit(entityId)),
      tap((r: IGetEntityReply) => {
        if (r.error && r.error.code == 406) {
          // @ts-ignore
          this.entity = null;
          // @ts-ignore
          this.diff = null;
          // @ts-ignore
          this.qualifications = null;
        } else if (r.error && r.error.code != 42) {
          // @ts-ignore
          this.entity = null;
          // @ts-ignore
          this.diff = [];
          // @ts-ignore
          this.qualifications = [];
          if (r.error.code != 42) {
            emitEditorErrorMessage(r.error.message);
          }
        } else {
          if (r.entity) {
            // @ts-ignore
            this.entity = r.entity;
            // @ts-ignore
            this.diff = r.diff;
            // @ts-ignore
            this.qualifications = r.qualifications;
          } else {
            // @ts-ignore
            this.entity = null;
            // @ts-ignore
            this.diff = null;
            // @ts-ignore
            this.qualifications = null;
          }
        }
      }),
    );

    return {
      entityQualificationStart: combineLatest(
        entityQualificationStart$,
        changeSet$,
        editSession$,
      ).pipe(
        tap(([qualificationStart, changeSet, editSession]) => {
          if (
            // @ts-ignore
            qualificationStart.entityId == this.selectedEntityId &&
            qualificationStart.changeSetId == changeSet?.id &&
            qualificationStart.editSessionId == editSession?.id
          ) {
            const newStart = _.unionBy(
              [qualificationStart],
              // @ts-ignore
              this.qualificationStart,
              "start",
            );
            // @ts-ignore
            this.qualificationStart = newStart;
          }
        }),
      ),
      entityQualifications: combineLatest(
        entityQualifications$,
        changeSet$,
        editSession$,
      ).pipe(
        tap(([qualification, changeSet, editSession]) => {
          if (
            // @ts-ignore
            qualification.entityId == this.selectedEntityId &&
            qualification.siChangeSet.changeSetId == changeSet?.id &&
            qualification.siChangeSet.editSessionId == editSession?.id
          ) {
            // @ts-ignore
            const newQuals = _.unionBy(
              [qualification],
              // @ts-ignore
              this.qualifications,
              "name",
            );
            // @ts-ignore
            this.qualifications = newQuals;

            const newStarts = _.filter(
              // @ts-ignore
              this.qualificationStart,
              q => q.start != qualification.name,
            );
            // @ts-ignore
            this.qualificationStart = newStarts;
          }
        }),
      ),
      attributePanelEntityUpdates: attributePanelEntityUpdates$.pipe(
        tap(reply => {
          if (reply.entity.id == this.$data.selectedEntityId) {
            // @ts-ignore
            this.entity = reply.entity;
            // @ts-ignore
            this.diff = reply.diff;
            // @ts-ignore
            this.qualifications = reply.qualifications;
          }
        }),
      ),
      entityLabelList: entityLabelList$.pipe(
        tap(r => {
          if (r.error && r.error.code != 42) {
            emitEditorErrorMessage(r.error.message);
          }
        }),
      ),
      schematicSelectNode: schematicSelectNode$.pipe(
        tap(node => {
          // @ts-ignore
          if (this.selectionIsLocked) {
            if (node) {
              // @ts-ignore
              this.selectedEntityId = node.object.id;
              // @ts-ignore
              this.selectedNode = node;
            } else {
              // @ts-ignore
              this.selectedEntityId = null;
              // @ts-ignore
              this.selectedNode = null;
            }
          }
        }),
      ),
      entityForEdit: entityForEdit$,
      resourceForEntity: combineLatest(
        system$,
        workspace$,
        selectedEntityId$,
      ).pipe(
        tap(async ([system, workspace, entityId]) => {
          if (system && workspace && entityId) {
            const request: IGetResourceRequest = {
              // @ts-ignore
              entityId,
              systemId: system.id,
              workspaceId: workspace.id,
            };
            const reply = await ResourceDal.getResource(request);
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              this.resource = reply.resource;
            }
          }
        }),
      ),
      resources: combineLatest(system$, selectedEntityId$, resources$).pipe(
        tap(([system, entityId, resource]) => {
          if (
            system &&
            entityId &&
            resource &&
            resource.systemId == system.id &&
            resource.entityId == entityId
          ) {
            this.resource = resource;
          }
        }),
      ),
      connectionsForEntity: selectedEntityId$.pipe(
        switchMap((entityId: string) => loadConnections(entityId)),
        tap((r: IGetConnectionsReply) => {
          const connections: Connections = {
            inbound: [],
            outbound: [],
          };

          if (r.error && r.error.code == 406) {
            // @ts-ignore
            this.connections = connections;
          } else if (r.error && r.error.code != 42) {
            // @ts-ignore
            this.connections = connections;
            if (r.error.code != 42) {
              emitEditorErrorMessage(r.error.message);
            }
          } else {
            if (r.connections) {
              // @ts-ignore
              this.connections = r.connections;
            } else {
              // @ts-ignore
              this.connections = connections;
            }
          }
        }),
      ),
      saveState: combineLatest(
        selectedEntityId$,
        selectionIsLocked$,
        activeView$,
      ).pipe(
        tap(([selectedEntityId, selectionIsLocked, activeView]) => {
          // TODO: fix this coupling to the router
          let applicationId = this.$route.params["applicationId"];
          attributePanelState$.next({
            panelRef: this.panelRef,
            applicationId,
            // @ts-ignore
            selectionIsLocked,
            // @ts-ignore
            selectedEntityId,
            // @ts-ignore
            activeView,
          });
        }),
      ),
      restoreState: restoreAttributePanelState$.pipe(
        tap(attributePanelState => {
          // TODO: fix this coupling to the router
          let applicationId = this.$route.params["applicationId"];
          if (
            this.panelRef == attributePanelState.panelRef &&
            applicationId == attributePanelState.applicationId
          ) {
            this.selectionIsLocked = attributePanelState.selectionIsLocked;
            if (!this.selectionIsLocked) {
              this.selectedEntityId = attributePanelState.selectedEntityId;
            }
            this.activeView = attributePanelState.activeView;
          }
        }),
      ),
    };
  },
  methods: {
    viewClasses(view: IData["activeView"]): Record<string, any> {
      if (view == this.activeView) {
        return { "menu-button-active": true };
      } else {
        return { "menu-button-inactive": true };
      }
    },
    attributeViewClasses(): Record<string, any> {
      return this.viewClasses("attribute");
    },
    codeViewClasses(): Record<string, any> {
      return this.viewClasses("code");
    },
    actionViewClasses(): Record<string, any> {
      return this.viewClasses("action");
    },
    eventViewClasses(): Record<string, any> {
      return this.viewClasses("event");
    },
    connectionViewClasses(): Record<string, any> {
      return this.viewClasses("connection");
    },
    resourceViewClasses(): Record<string, any> {
      return this.viewClasses("resource");
    },
    qualificationViewClasses(): Record<string, any> {
      return this.viewClasses("qualification");
    },
    switchToCodeView() {
      this.activeView = "code";
    },
    switchToAttributeView() {
      this.activeView = "attribute";
    },
    switchToEventView() {
      this.activeView = "event";
    },
    switchTocCnnectionView() {
      this.activeView = "connection";
    },
    switchToQualificationView() {
      this.activeView = "qualification";
    },
    switchToActionView() {
      this.activeView = "action";
    },
    switchToResourceView() {
      this.activeView = "resource";
    },
    async toggleSelectionLock() {
      if (this.selectionIsLocked) {
        this.selectionIsLocked = false;
      } else {
        this.selectionIsLocked = true;
      }
    },
  },
  mounted() {
    refreshEntityLabelList$.next(true);
  },
});
</script>

<style scoped>
.menu-button-active {
  color: #69e3d2;
}

.menu-button-inactive {
  color: #c6c6c6;
}

.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
