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
      <div class="flex w-20">
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
      <button class="pl-1 focus:outline-none" @click="toggleSelectionLock()">
        <UnlockIcon size="1.1x" v-if="selectionIsLocked" />
        <LockIcon size="1.1x" v-else />
      </button>
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
      <button
        class="pl-1 text-white focus:outline-none"
        :class="actionViewClasses()"
        @click="switchToActionView()"
      >
        <PlayIcon size="1.1x" />
      </button>

      <button
        class="pl-1 text-white focus:outline-none"
        :class="eventViewClasses()"
        @click="switchToEventView()"
      >
        <RadioIcon size="1.1x" />
      </button>

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
    </template>
    <template v-slot:content>
      <div class="flex flex-row w-full h-full" v-if="entity">
        <AttributeViewer
          v-if="activeView == 'attribute'"
          :entity="entity"
          :diff="diff"
        />
        <QualificationViewer
          v-else-if="activeView == 'qualification'"
          :entity="entity"
          :qualifications="qualifications"
          :starting="qualificationStart"
        />
        <ActionViewer v-else-if="activeView == 'action'" :entity="entity" />

        <div v-else>
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
  RadioIcon,
  DiscIcon,
  CheckSquareIcon,
  PlayIcon,
} from "vue-feather-icons";
import "vue-json-pretty/lib/styles.css";
import { Entity } from "@/api/sdf/model/entity";
import Bottle from "bottlejs";
import { Persister } from "@/api/persister";
import AttributeViewer from "@/organisims/AttributeViewer.vue";
import QualificationViewer from "@/organisims/QualificationViewer.vue";
import ActionViewer from "@/organisims/ActionViewer.vue";
//import CodeViewer from "@/organisims/CodeViewer.vue";
import {
  loadEntityForEdit,
  attributePanelEntityUpdates$,
  entityLabelList$,
  schematicSelectedEntityId$,
  entityQualifications$,
  entityQualificationStart$,
  changeSet$,
  editSession$,
  refreshEntityLabelList$,
} from "@/observables";
import { combineLatest } from "rxjs";
import { pluck, switchMap, tap, filter, map } from "rxjs/operators";
import { Diff } from "@/api/sdf/model/diff";
import {
  Qualification,
  QualificationStart,
} from "@/api/sdf/model/qualification";
import _ from "lodash";

interface IData {
  isLoading: boolean;
  selectedEntityId: string;
  selectionIsLocked: boolean;
  activeView: "attribute" | "code" | "event" | "qualification" | "action";
  entity: Entity | null;
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
    RadioIcon,
    CodeIcon,
    LockIcon,
    UnlockIcon,
    CheckSquareIcon,
    AttributeViewer,
    QualificationViewer,
    PlayIcon,
    ActionViewer,
  },
  data(): IData {
    let bottle = Bottle.pop("default");
    let persister: Persister = bottle.container.Persister;
    let persistedData = persister.getData(`${this.panelRef}-data`);
    if (persistedData) {
      if (persistedData["entity"]) {
        persistedData["entity"] = Entity.fromJson(persistedData["entity"]);
      }
      return persistedData;
    } else {
      return {
        isLoading: false,
        selectedEntityId: "",
        selectionIsLocked: true,
        activeView: "attribute",
        entity: null,
        diff: [],
        qualifications: [],
        qualificationStart: [],
      };
    }
  },
  subscriptions(): any {
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
      schematicSelectedEntityId: schematicSelectedEntityId$.pipe(
        tap(entityId => {
          // @ts-ignore
          if (this.selectionIsLocked) {
            // @ts-ignore
            this.selectedEntityId = entityId;
          }
        }),
      ),
      entityForEdit: this.$watchAsObservable("selectedEntityId", {
        immediate: true,
      }).pipe(
        pluck("newValue"),
        switchMap(entityId => loadEntityForEdit(entityId)),
        tap(r => {
          if (r.error && r.error.code != 42) {
            // @ts-ignore
            this.entity = null;
            // @ts-ignore
            this.diff = [];
            // @ts-ignore
            this.qualifications = [];
            emitEditorErrorMessage(r.error.message);
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
      ),
    };
  },
  methods: {
    viewClasses(view: IData["activeView"]): Record<string, any> {
      if (view == this.activeView) {
        return { "text-blue-300": true };
      } else {
        return {};
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
    switchToQualificationView() {
      this.activeView = "qualification";
    },
    switchToActionView() {
      this.activeView = "action";
    },
    async toggleSelectionLock() {
      if (this.selectionIsLocked) {
        this.selectionIsLocked = false;
      } else {
        this.selectionIsLocked = true;
      }
    },
  },
  async beforeDestroy() {
    let bottle = Bottle.pop("default");
    let persister: Persister = bottle.container.Persister;
    persister.removeData(`${this.panelRef}-data`);
  },
  watch: {
    $data: {
      handler: function(newData, _oldData) {
        let bottle = Bottle.pop("default");
        let persister: Persister = bottle.container.Persister;
        persister.setData(`${this.panelRef}-data`, newData);
      },
      deep: true,
    },
  },
  mounted() {
    refreshEntityLabelList$.next(true);
  },
});
</script>
