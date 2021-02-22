<template>
  <div
    class="flex flex-row mt-2 items-top"
    v-if="currentValue && (Object.keys(currentValue).length || editMode)"
  >
    <div
      class="w-40 px-2 text-sm leading-tight text-right text-white input-label"
    >
      {{ registryProperty.name }}
    </div>

    <div class="w-4/5 ml-2">
      <div
        v-for="[key, value] of Object.entries(currentValue)"
        :key="key"
        class="flex pb-2"
      >
        <div class="flex w-full row" v-if="!editMode">
          <div
            class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
            v-bind:class="mapTextClasses(key)"
          >
            {{ key }}: {{ value }}
          </div>
        </div>
        <div class="flex flex-row items-center" v-else>
          <div
            class="w-2/5 pl-2 text-sm leading-tight text-gray-400"
            v-if="key"
          >
            {{ key }}:
          </div>
          <input
            class="w-3/5 pl-2 ml-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
            v-bind:class="mapInputClasses(key)"
            type="text"
            aria-label="val"
            v-model="currentValue[key]"
            placeholder="value"
            @blur="saveIfModified()"
          />

          <button
            class="pl-1 text-gray-600 focus:outline-none"
            type="button"
            @click="removeFromMap(key)"
          >
            <x-icon size="0.8x"></x-icon>
          </button>
        </div>
      </div>
      <div v-if="hasNew" class="flex pb-2">
        <div class="items-center">
          <input
            class="w-2/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
            type="text"
            aria-label="key"
            v-model="newKey"
            placeholder="key"
          />
          <input
            class="w-2/5 pl-2 ml-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
            type="text"
            aria-label="val"
            v-model="newValue"
            placeholder="value"
            :disabled="!newKey"
            @blur="addNew()"
          />
          <button
            class="pl-1 text-gray-600 focus:outline-none"
            type="button"
            @click="cancelNew"
          >
            <x-icon size="0.8x"></x-icon>
          </button>
        </div>
      </div>

      <div class="flex text-gray-500" v-if="editMode">
        <button class="focus:outline-none" type="button" @click="addToMap">
          <plus-square-icon size="1.25x"></plus-square-icon>
        </button>
      </div>
      <ValidationWidget
        :value="currentValue"
        :registryProperty="registryProperty"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Store, mapState, mapGetters } from "vuex";
import { PlusSquareIcon, XIcon } from "vue-feather-icons";
import _ from "lodash";
import {
  IEntitySetPropertyReply,
  IEntitySetPropertyRequest,
} from "@/api/sdf/dal/editorDal";
import {
  emitEditorErrorMessage,
  onPropChangeEvent,
  IPropChangeEvent,
  offPropChangeEvent,
  onPropMapAddEvent,
  offPropMapAddEvent,
  emitPropMapAddEvent,
  onPropMapRemoveEvent,
  offPropMapRemoveEvent,
  emitPropMapRemoveEvent,
} from "@/atoms/PanelEventBus";
import { SessionStore } from "@/store/modules/session";
import { EditorStore } from "@/store/modules/editor";
import { RegistryProperty } from "@/api/registryProperty";

import ValidationWidget from "@/atoms/ValidationWidget.vue";

interface IData {
  currentValue: Record<string, any> | undefined;
  startingValue: Record<string, any> | undefined;
  projectionValue: Record<string, any> | undefined;
  baseValue: Record<string, any> | undefined;
  isBeingEdited: boolean;
  save: boolean;
  newKey: string;
  newValue: string;
  hasNew: boolean;
}

export default Vue.extend({
  name: "PropMap",
  props: {
    entityId: {
      type: String,
      required: true,
    },
    registryProperty: {
      type: Object as PropType<RegistryProperty>,
      required: true,
    },
    // The value of the property's head (if it has been merged) or base (if it was created in this change set)
    initialBaseValue: {
      type: Object,
    },
    // The initial value of the property in this changeSet (either the head value or its current projection)
    initialProjectionValue: {
      type: Object,
    },
  },
  data(): IData {
    let baseValue;
    let projectionValue;
    let currentValue;
    if (this.initialBaseValue) {
      baseValue = _.cloneDeep(this.initialBaseValue);
    } else {
      baseValue = undefined;
    }
    if (this.initialProjectionValue) {
      projectionValue = _.cloneDeep(this.initialProjectionValue);
    } else {
      projectionValue = undefined;
    }
    currentValue = projectionValue;
    return {
      currentValue,
      projectionValue,
      baseValue,
      startingValue: undefined,
      isBeingEdited: false,
      save: false,
      newKey: "",
      newValue: "",
      hasNew: false,
    };
  },
  components: {
    PlusSquareIcon,
    XIcon,
    ValidationWidget,
  },
  computed: {
    editMode(): boolean {
      return this.$store.getters["editor/inEditable"];
    },
    ...mapState({
      currentSystem: (state: any): SessionStore["currentSystem"] =>
        state.session.currentSystem,
      currentWorkspace: (state: any): SessionStore["currentWorkspace"] =>
        state.session.currentWorkspace,
      currentChangeSet: (state: any): EditorStore["currentChangeSet"] =>
        state.editor.currentChangeSet,
      currentEditSession: (state: any): EditorStore["currentEditSession"] =>
        state.editor.currentEditSession,
    }),
    textClasses(): Record<string, boolean> {
      let results: Record<string, boolean> = {};
      if (this.hasBeenEdited) {
        results["input-border-gold"] = true;
        results["border"] = true;
      } else {
        results["input-border-grey"] = true;
      }
      return results;
    },
    inputClasses(): Record<string, boolean> {
      let results: Record<string, boolean> = {};
      results["si-property"] = true;
      if (this.hasBeenEdited) {
        results["input-border-gold"] = true;
        results["input-bg-color-grey"] = true;
      } else {
        results["input-border-grey"] = true;
        results["input-bg-color-grey"] = true;
      }
      return results;
    },
    hasBeenEdited(): boolean {
      return this.baseValue != this.currentValue;
    },
  },
  methods: {
    async addNew() {
      if (this.hasNew && this.newKey && this.newValue && this.currentValue) {
        Vue.set(this.currentValue, this.newKey, this.newValue);
        let path = _.cloneDeep(this.registryProperty.path);
        path.push(this.newKey);
        console.log("adding new and emitting", {
          path,
          entityId: this.entityId,
          addMapValue: this.newValue,
        });
        emitPropMapAddEvent({ path }, this.entityId, "add", this.newValue);
        onPropChangeEvent({ path }, this.entityId, this.onPropChange);
        await this.saveIfModified();
      }
      this.hasNew = false;
      this.newKey = "";
      this.newValue = "";
    },
    cancelNew() {
      this.hasNew = false;
      this.newKey = "";
      this.newValue = "";
    },
    addToMap(): void {
      this.newKey = "";
      this.newValue = "";
      this.hasNew = true;
    },
    async removeFromMap(key: string): Promise<void> {
      if (this.currentValue) {
        Vue.delete(this.currentValue, key);
        let path = _.cloneDeep(this.registryProperty.path);
        path.push(key);
        emitPropMapRemoveEvent({ path }, this.entityId, "delete", undefined);
        offPropChangeEvent({ path }, this.entityId, this.onPropChange);
        await this.saveIfModified();
      }
    },
    storeStartingValue() {
      this.isBeingEdited = true;
    },
    async saveIfModified(): Promise<void> {
      if (
        this.currentSystem &&
        this.currentWorkspace &&
        this.currentEditSession &&
        this.currentChangeSet
      ) {
        if (!_.isEqual(this.currentValue, this.startingValue)) {
          let reply: IEntitySetPropertyReply;
          let request: IEntitySetPropertyRequest = {
            workspaceId: this.currentWorkspace.id,
            entityId: this.entityId,
            changeSetId: this.currentChangeSet.id,
            editSessionId: this.currentEditSession.id,
            path: this.registryProperty.path,
            value: this.currentValue,
          };
          reply = await this.$store.dispatch(
            "editor/entitySetProperty",
            request,
          );
          if (reply.error) {
            emitEditorErrorMessage(reply.error.message);
          }
        }
      }
      this.isBeingEdited = false;
    },
    onPropChange(event: IPropChangeEvent) {
      if (!this.isBeingEdited && this.currentValue) {
        Vue.set(
          this.currentValue,
          event.registryProperty.path[event.registryProperty.path.length - 1],
          _.cloneDeep(event.value),
        );
      }
    },
    onPropMapAdd(event: IPropChangeEvent) {
      if (!this.isBeingEdited && this.currentValue) {
        Vue.set(
          this.currentValue,
          event.registryProperty.path[event.registryProperty.path.length - 1],
          _.cloneDeep(event.value),
        );
        onPropChangeEvent(
          event.registryProperty,
          this.entityId,
          this.onPropChange,
        );
      }
    },
    onPropMapRemove(event: IPropChangeEvent) {
      if (!this.isBeingEdited && this.currentValue) {
        Vue.delete(
          this.currentValue,
          event.registryProperty.path[event.registryProperty.path.length - 1],
        );
        offPropChangeEvent(
          event.registryProperty,
          this.entityId,
          this.onPropChange,
        );
      }
    },
    mapTextClasses(key: string): Record<string, boolean> {
      let results: Record<string, boolean> = {};
      if (this.mapHasBeenEdited(key)) {
        results["input-border-gold"] = true;
        results["border"] = true;
      } else {
        results["input-border-grey"] = true;
      }
      return results;
    },
    mapInputClasses(key: string): Record<string, boolean> {
      let results: Record<string, boolean> = {};
      results["si-property"] = true;
      if (this.mapHasBeenEdited(key)) {
        results["input-border-gold"] = true;
        results["input-bg-color-grey"] = true;
      } else {
        results["input-border-grey"] = true;
        results["input-bg-color-grey"] = true;
      }
      return results;
    },
    mapHasBeenEdited(key: string): boolean {
      if (this.baseValue && this.currentValue) {
        return this.baseValue[key] != this.currentValue[key];
      } else {
        return false;
      }
      //const path = _.cloneDeep(this.entityProperty.path);
      //path.push(key);

      //let result = _.find(this.diff.entries, diffEntry => {
      //  return _.isEqual(
      //    diffEntry.path,
      //    ["properties", "__baseline"].concat(path),
      //  );
      //});
      //if (result) {
      //  return true;
      //} else {
      //  return false;
      //}
    },
  },
  mounted() {
    if (this.currentValue) {
      for (const key of Object.keys(this.currentValue)) {
        let path = _.cloneDeep(this.registryProperty.path);
        path.push(key);
        onPropChangeEvent({ path }, this.entityId, this.onPropChange);
      }
    }
    onPropMapAddEvent(this.registryProperty, this.entityId, this.onPropMapAdd);
    onPropMapRemoveEvent(
      this.registryProperty,
      this.entityId,
      this.onPropMapRemove,
    );
  },
  beforeDestroy() {
    if (this.currentValue) {
      for (const key of Object.keys(this.currentValue)) {
        let path = _.cloneDeep(this.registryProperty.path);
        path.push(key);
        offPropChangeEvent({ path }, this.entityId, this.onPropChange);
      }
    }
    offPropMapAddEvent(this.registryProperty, this.entityId, this.onPropMapAdd);
    offPropMapRemoveEvent(
      this.registryProperty,
      this.entityId,
      this.onPropMapRemove,
    );
  },
});
</script>

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}
</style>
