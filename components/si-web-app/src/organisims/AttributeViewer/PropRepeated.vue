<template>
  <section :class="accordionClasses" v-if="currentValue || editMode">
    <div
      class="pl-2 text-sm text-white cursor-pointer section-header"
      @click="togglePath(registryProperty.path)"
    >
      <div v-if="isOpen" class="flex" :style="propObjectStyle">
        <chevron-down-icon size="1.5x"></chevron-down-icon>
        {{ registryProperty.name }}
      </div>

      <div v-else-if="!isOpen" class="flex" :style="propObjectStyle">
        <chevron-right-icon size="1.5x"></chevron-right-icon>
        {{ registryProperty.name }}
      </div>
    </div>
    <div>
      <div class="flex flex-col text-gray-400">
        <div
          class="flex flex-row items-center pl-2 ml-16 mr-12"
          v-for="(repeatedEntry, index) in currentValue"
          :key="index"
          v-show="showPath(registryProperty, index)"
        >
          <div class="w-full mb-2 border rounded-sm repeated-border">
            <div
              class="text-white"
              v-for="ep in getPropertiesListRepeated(registryProperty, index)"
              :key="ep.id"
            >
              <div v-if="!ep.hidden" class="flex flex-row">
                <div class="w-full" :style="propStyle(ep)">
                  <div class="py-1">
                    <div v-if="repeated(ep)">
                      <PropRepeated
                        :registryProperty="ep"
                        :isOpen="checkIsOpen(ep)"
                        :backgroundColors="backgroundColors"
                        :collapsedPaths="collapsedPaths"
                        :entityId="currentObject.id"
                        :initialBaseValue="getInitialBaseValue(ep)"
                        :initialProjectionValue="getInitialProjectionValue(ep)"
                        :currentObject="currentObject"
                        :baseObject="baseObject"
                        :attributeStoreCtx="attributeStoreCtx"
                        @toggle-path="togglePath(ep.path)"
                      />
                    </div>

                    <div v-else-if="propKind(ep, 'object')">
                      <PropObject
                        :registryProperty="ep"
                        :isOpen="checkIsOpen(ep)"
                        :editObject="currentObject"
                        @toggle-path="togglePath(ep.path)"
                      />
                    </div>

                    <div v-else-if="propKind(ep, 'text')">
                      <PropText
                        :registryProperty="ep"
                        :entityId="entityId"
                        :initialBaseValue="getInitialBaseValue(ep)"
                        :initialProjectionValue="getInitialProjectionValue(ep)"
                      />
                    </div>

                    <div v-else-if="propKind(ep, 'code')">
                      <!-- for now, do nothing! -->
                    </div>

                    <div v-else-if="propKind(ep, 'number')">
                      <PropNumber
                        :registryProperty="ep"
                        :entityId="entityId"
                        :initialBaseValue="getInitialBaseValue(ep)"
                        :initialProjectionValue="getInitialProjectionValue(ep)"
                      />
                    </div>

                    <div v-else-if="propKind(ep, 'enum')">
                      <PropEnum
                        :registryProperty="ep"
                        :entityId="entityId"
                        :initialBaseValue="getInitialBaseValue(ep)"
                        :initialProjectionValue="getInitialProjectionValue(ep)"
                      />
                    </div>

                    <div v-else-if="propKind(ep, 'map')">
                      <PropMap
                        :registryProperty="ep"
                        :entityId="entityId"
                        :initialBaseValue="getInitialBaseValue(ep)"
                        :initialProjectionValue="getInitialProjectionValue(ep)"
                      />
                    </div>

                    <div v-else-if="propKind(ep, 'select')">
                      <PropSelect
                        :registryProperty="ep"
                        :entityId="entityId"
                        :initialBaseValue="getInitialBaseValue(ep)"
                        :initialProjectionValue="getInitialProjectionValue(ep)"
                      />
                    </div>

                    <div v-else class="text-red-700">
                      {{ ep.name }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div v-show="showPath(registryProperty, index) && editMode">
            <button
              class="pl-1 text-gray-600 focus:outline-none"
              type="button"
              @click="removeFromList(index)"
            >
              <x-icon size="0.8x"></x-icon>
            </button>
          </div>
        </div>
        <div>
          <div
            class="flex justify-center text-gray-500 align-middle"
            v-if="editMode"
          >
            <button
              class="w-4 pl-4 text-center focus:outline-none"
              type="button"
              @click="addToList"
            >
              <PlusSquareIcon size="1.25x" class=""></PlusSquareIcon>
            </button>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Store, mapState, mapGetters } from "vuex";
import {
  ChevronDownIcon,
  ChevronRightIcon,
  PlusSquareIcon,
  XIcon,
} from "vue-feather-icons";
import _ from "lodash";

import { InstanceStoreContext, RootStore } from "@/store";
import PropText from "./PropText.vue";
import PropObject from "./PropObject.vue";
import PropNumber from "./PropNumber.vue";
import PropEnum from "./PropEnum.vue";
import PropMap from "./PropMap.vue";
import PropSelect from "./PropSelect.vue";

import { RegistryProperty, propertyListRepeated } from "@/api/registryProperty";
import { SessionStore } from "@/store/modules/session";
import { EditorStore } from "@/store/modules/editor";
import { AttributeStore } from "@/store/modules/attribute";
import { Entity } from "@/api/sdf/model/entity";
import {
  IEntitySetPropertyReply,
  IEntitySetPropertyRequest,
} from "@/api/sdf/dal/editorDal";
import {
  emitEditorErrorMessage,
  onPropRepeatedAddEvent,
  offPropRepeatedAddEvent,
  emitPropRepeatedAddEvent,
  onPropRepeatedRemoveEvent,
  offPropRepeatedRemoveEvent,
  emitPropRepeatedRemoveEvent,
  IPropChangeEvent,
  onPropChangeEvent,
} from "@/atoms/PanelEventBus";

// This component only works with repeated objects! When we need it to work with
// repeated fields of other types, we're going to have to extend it. For now,
// no entity has a repeated but non-object field.
interface IData {
  currentValue: Record<string, any>[] | undefined;
  startingValue: Record<string, any>[] | undefined;
  projectionValue: Record<string, any>[] | undefined;
  baseValue: Record<string, any>[] | undefined;
  isBeingEdited: boolean;
}

export default Vue.extend({
  name: "PropRepeated",
  components: {
    PropText,
    PropObject,
    PropNumber,
    PropEnum,
    PropMap,
    ChevronRightIcon,
    ChevronDownIcon,
    PlusSquareIcon,
    XIcon,
  },
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
      type: Array as PropType<Record<string, any>[]>,
    },
    // The initial value of the property in this changeSet (either the head value or its current projection)
    initialProjectionValue: {
      type: Array as PropType<Record<string, any>[]>,
    },
    baseObject: {
      type: Object as PropType<Entity>,
    },
    currentObject: {
      type: Object as PropType<Entity>,
    },
    backgroundColors: Array,
    collapsedPaths: Array,
    isOpen: Boolean,
    attributeStoreCtx: Object as PropType<InstanceStoreContext<AttributeStore>>,
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
    };
  },
  computed: {
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
    editMode(): boolean {
      return this.$store.getters["editor/inEditable"];
    },
    propObjectStyle(): string {
      if (this.registryProperty.path.length == 1) {
        return "";
      }
      let results = `margin-left: ${this.registryProperty.path.length * 10}px`;
      return results;
    },
    accordionClasses(): { "is-closed": boolean } {
      return {
        "is-closed": !this.isOpen,
        // 'is-primary': this.isOpen,
        // 'is-dark': !this.isOpen
      };
    },
  },
  methods: {
    getInitialBaseValue(
      registryProperty: RegistryProperty,
    ): string | number | undefined {
      return _.get(this.baseObject, registryProperty.path);
    },
    getInitialProjectionValue(
      registryProperty: RegistryProperty,
    ): string | number | undefined {
      return _.get(this.currentObject, registryProperty.path);
    },
    getPropertiesListRepeated(
      registryProperty: RegistryProperty,
      index: number,
    ): RegistryProperty[] {
      if (this.attributeStoreCtx.state.currentObject) {
        return propertyListRepeated(
          this.attributeStoreCtx.state.currentObject,
          registryProperty,
          index,
        );
      } else {
        return [];
      }
    },
    togglePath(path: (string | number)[]) {
      this.$emit("toggle-path", path);
    },
    propKind(prop: RegistryProperty, kindToCheck: string): boolean {
      return prop.kind == kindToCheck;
    },
    repeated(prop: RegistryProperty): boolean {
      return prop.repeated;
    },
    addToList(): void {
      if (!this.currentValue) {
        this.currentValue = Vue.observable([]);
      }
      this.currentValue.push(Vue.observable({}));
      console.log("added to list", {
        currentValue: JSON.stringify(this.currentValue),
      });
      //this.saveIfModified();
      emitPropRepeatedAddEvent(this.registryProperty, this.entityId, "add", {
        from: this.attributeStoreCtx.dispatchPath(),
      });
    },
    removeFromList(index: number): void {
      let current = _.cloneDeep(this.currentValue);
      if (current) {
        current.splice(index, 1);
        this.currentValue = current;
      }
      this.saveIfModified();
      emitPropRepeatedRemoveEvent(
        this.registryProperty,
        this.entityId,
        "delete",
        {
          index: index,
          from: this.attributeStoreCtx.dispatchPath(),
        },
      );
    },
    showPath(prop: RegistryProperty, index: number): boolean {
      let propPath = _.cloneDeep(prop.path);
      propPath.push(`${index}`);
      const collapsed = _.find(
        this.collapsedPaths,
        (path: (string | number)[]) => {
          if (propPath.length >= path.length) {
            if (!_.isEmpty(index)) {
              if (_.isEqual(propPath.slice(0, propPath.length - 1), path)) {
                // We always want to show the toggle path!
                return false;
              }
            } else {
              if (_.isEqual(propPath, path)) {
                // We always want to show the toggle path!
                return false;
              }
            }
            const sliceToCheck = propPath.slice(0, path.length);
            return _.isEqual(sliceToCheck, path);
          } else {
            return false;
          }
        },
      );
      if (collapsed) {
        return false;
      } else {
        return true;
      }
    },
    propStyle(registryProperty: RegistryProperty): string {
      let rgb: number[];
      if (
        registryProperty.name == "properties" &&
        registryProperty.path.length == 1
      ) {
        return "";
      } else {
        let maxDepth = this.backgroundColors.length;
        let epDepth = registryProperty.path.length;
        let depth;
        if (epDepth > maxDepth) {
          depth = maxDepth - 1;
        } else {
          depth = epDepth - 1;
        }
        // @ts-ignore
        rgb = this.backgroundColors[depth];
      }
      return `background-color: rgb(${rgb.join(",")});`;
    },
    checkIsOpen(prop: RegistryProperty): boolean {
      const collapsed = _.find(this.collapsedPaths, path => {
        if (_.isEqual(prop.path, path)) {
          return true;
        } else {
          return false;
        }
      });
      if (collapsed) {
        return false;
      } else {
        return true;
      }
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
    onPropRepeatedAdd(event: IPropChangeEvent) {
      if (!this.currentValue) {
        this.currentValue = [];
      }
      if (!this.isBeingEdited) {
        // @ts-ignore - we know we're abusing this field. it's okay. don't freak out.
        if (event.value["from"] != this.attributeStoreCtx.dispatchPath()) {
          let current = _.cloneDeep(this.currentValue);
          current.push(Vue.observable({}));
          this.currentValue = current;
        }
      }
    },
    onPropRepeatedRemove(event: IPropChangeEvent) {
      if (!this.isBeingEdited && !_.isUndefined(this.currentValue)) {
        // @ts-ignore - we know we're abusing this field. it's okay. don't freak out.
        if (event.value["from"] != this.attributeStoreCtx.dispatchPath()) {
          let current = _.cloneDeep(this.currentValue);
          if (current) {
            // @ts-ignore
            current.splice(event.value["index"], 1);
            this.currentValue = current;
          }
        }
      }
    },
  },
  mounted(): void {
    // TODO: the belief is that the repeated child cannot recieve new events,
    // because it doesn't exist yet. Something that does needs to watch also.
    onPropRepeatedAddEvent(
      this.registryProperty,
      this.entityId,
      this.onPropRepeatedAdd,
    );
    onPropRepeatedRemoveEvent(
      this.registryProperty,
      this.entityId,
      this.onPropRepeatedRemove,
    );
  },
});
</script>

<style scoped>
.property-section-title-bg-color {
  background-color: #292c2d;
}
.repeated-border {
  border-color: #2b2e2f;
}
</style>
