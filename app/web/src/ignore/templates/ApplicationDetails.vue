<template>
  <div id="application-editor" class="flex flex-col w-full h-full select-none">
    <div class="flex flex-col w-full h-full">
      <StatusBar :application="application" />
      <ApplicationContext
        :workspaceId="workspaceId"
        :application="application"
      />
      <div id="editor" class="flex w-full h-full overflow-hidden">
        <Editor :context="editorContext" />
      </div>
      <EventBar />
      <!--
    <div id="eventBar" class="w-full">
      <EventBar />
    </div>
      -->
    </div>
    <!-- this one is extra -->

    <SiModal name="leave" title="Alert" class="">
      <div class="flex flex-col items-center w-full h-full mb-2">
        <div class="text-base font-normal text-red-500">
          You have unsaved changes!
        </div>
        <div class="text-sm text-white">Are you sure you want to leave?</div>
      </div>
      <template v-slot:buttons>
        <SiButton
          size="sm"
          label="leave"
          class="mx-1"
          icon="null"
          kind="cancel"
          @click.native="leave"
        />
        <SiButton
          size="sm"
          label="stay"
          class="mx-1"
          icon="null"
          kind="save"
          @click.native="stay"
        />
      </template>
    </SiModal>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import StatusBar from "@/organisims/StatusBar.vue";
import EventBar from "@/organisims/EventBar.vue";
import ApplicationContext from "@/organisims/ApplicationContext.vue";
import Editor from "@/organisims/Editor.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiButton from "@/atoms/SiButton.vue";

export interface IEditorContextApplication {
  applicationId: string;
  contextType: "applicationSystem";
}

import {
  system$,
  editMode$,
  workspace$,
  getEntity,
  applicaton$,
  applicationId$,
  changeSet$,
  editSession$,
  deploymentSchematicSelectNode$,
  schematicSelectNode$,
} from "@/observables";
import { combineLatest } from "rxjs";
import { tap, switchMap } from "rxjs/operators";
import { IEntity } from "@/api/sdf/model/entity";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface IData {
  application: IEntity | null;
  navDestination: any | null;
}

export default Vue.extend({
  name: "ApplicationDetails",
  components: {
    StatusBar,
    EventBar,
    ApplicationContext,
    Editor,
    SiModal,
    SiButton,
  },
  data(): IData {
    return {
      navDestination: null,
      application: null,
    };
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
    applicationId: {
      type: String,
    },
  },
  subscriptions(this: any): Record<string, any> {
    return {
      currentSystem: system$,
      editMode: editMode$,
      currentApplication: combineLatest(workspace$).pipe(
        switchMap(([workspace]) =>
          getEntity(this.applicationId, workspace, null, null),
        ),
        tap(result => {
          if (result.error) {
            if (result.error.code != 42) {
              emitEditorErrorMessage(result.error.message);
            }
          } else {
            this.application = result.entity;
          }
        }),
      ),
    };
  },
  computed: {
    editorContext(): IEditorContextApplication {
      return {
        applicationId: this.applicationId,
        contextType: "applicationSystem",
      };
    },
  },
  methods: {
    leave() {
      this.$modal.hide("leave");
      this.cleanUpState();
      this.navDestination();
    },
    cleanUpState() {
      applicaton$.next(null);
      applicationId$.next(null);
      changeSet$.next(null);
      editSession$.next(null);
      editMode$.next(false);
      deploymentSchematicSelectNode$.next(null);
      schematicSelectNode$.next(null);
      sessionStorage.removeItem("schematicPanelKind$");
      sessionStorage.removeItem("panelTypeChanges$");
    },
    stay() {
      this.$modal.hide("leave");
    },
  },
  async created() {},
  beforeRouteLeave(this: any, _to, _from, next: any) {
    if (this.editMode) {
      if (next != null) {
        this.navDestination = next;
        this.$modal.show("leave");
        next(false);
      }
    } else {
      this.cleanUpState();
      next();
    }
  },
});
</script>
