import Vue, { PropType } from "vue";
import { mapState } from "vuex";
import { RegistryProperty } from "@/api/registryProperty";
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
} from "@/atoms/PanelEventBus";
import { SessionStore } from "@/store/modules/session";
import { EditorStore } from "@/store/modules/editor";

interface IData {
  currentValue: string | number | undefined;
  startingValue: string | number | undefined;
  projectionValue: string | number | undefined;
  baseValue: string | number | undefined;
  isBeingEdited: boolean;
}

export default Vue.extend({
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
      type: [String, Number],
    },
    // The initial value of the property in this changeSet (either the head value or its current projection)
    initialProjectionValue: {
      type: [String, Number],
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
    };
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
    storeStartingValue() {
      this.startingValue = _.cloneDeep(this.currentValue);
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
          if (this.registryProperty.kind == "number" && this.currentValue) {
            let fieldValue: number;
            if (typeof this.currentValue == "string") {
              fieldValue = parseInt(this.currentValue, 10);
            } else {
              fieldValue = this.currentValue;
            }
            let request: IEntitySetPropertyRequest = {
              workspaceId: this.currentWorkspace.id,
              entityId: this.entityId,
              changeSetId: this.currentChangeSet.id,
              editSessionId: this.currentEditSession.id,
              path: this.registryProperty.path,
              value: fieldValue,
            };
            reply = await this.$store.dispatch(
              "editor/entitySetProperty",
              request,
            );
          } else {
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
          }
          if (reply.error) {
            emitEditorErrorMessage(reply.error.message);
          }
        }
      }
      this.isBeingEdited = false;
    },
    onPropChange(event: IPropChangeEvent) {
      if (!this.isBeingEdited) {
        // @ts-ignore
        this.currentValue = event.value;
      }
    },
  },
  created() {
    onPropChangeEvent(this.registryProperty, this.entityId, this.onPropChange);
  },
  beforeDestroy() {
    offPropChangeEvent(this.registryProperty, this.entityId, this.onPropChange);
  },
});
