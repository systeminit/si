<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";

import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import {
  EditField,
  OpSet,
  OpType,
  OpSource,
  OpUnset,
  OpTombstone,
} from "si-entity/dist/siEntity";

import { updateEntity } from "@/observables";
import { Entity } from "@/api/sdf/model/entity";
import { ValidateFailure } from "si-entity/dist/validation";

interface Data {
  startValue: unknown | undefined;
  currentValue: unknown | undefined;
  updating: boolean;
  errors: ValidateFailure["errors"];
}

export default Vue.extend({
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    editMode: {
      type: Boolean,
      required: true,
    },
    editField: {
      type: Object as PropType<EditField>,
      required: true,
    },
    systemId: {
      type: String,
    },
  },
  data(): Data {
    return {
      startValue: null,
      currentValue: null,
      updating: false,
      errors: [],
    };
  },
  computed: {
    showField(): boolean {
      return this.editMode || !_.isUndefined(this.currentValue);
    },
    fieldName(): string {
      if (this.editField) {
        return this.editField.name;
      } else {
        return "schema bug!";
      }
    },
    isDisabled(): boolean {
      if (this.entity) {
        return this.entity.isTombstoned({
          path: this.editField.path,
          system: this.systemId,
          source: OpSource.Manual,
        });
      }
      return false;
    },
  },
  methods: {
    async toggleTombstone(
      {
        source,
        system,
      }: {
        source: OpTombstone["source"];
        system?: OpTombstone["system"];
      },
      editField?: EditField,
    ) {
      if (!editField) {
        editField = this.editField;
      }
      if (this.entity) {
        let systemId;
        if (system) {
          systemId = system;
        } else {
          systemId = this.systemId;
        }
        if (
          this.hasTombstone({
            source,
            path: editField.path,
            system: systemId,
          })
        ) {
          const opTombstone: OpTombstone = {
            op: OpType.Tombstone,
            source,
            path: editField.path,
            system: systemId,
          };
          this.entity.removeOpTombstone(opTombstone);
          this.entity.computeProperties();
          updateEntity(this.entity).subscribe(reply => {
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            }
          });
        } else {
          const opTombstone: OpTombstone = {
            op: OpType.Tombstone,
            source,
            path: editField.path,
            system: systemId,
          };
          this.entity.addOpTombstone(opTombstone);
          this.entity.computeProperties();
          updateEntity(this.entity).subscribe(reply => {
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            }
          });
        }
        this.updateOnPropChanges();
      }
    },
    hasTombstone({
      source,
      path,
      system,
    }: {
      source: OpTombstone["source"];
      path: OpTombstone["path"];
      system: OpTombstone["system"];
    }): boolean {
      if (this.entity) {
        return this.entity.isTombstoned({ source, path, system });
      } else {
        return false;
      }
    },
    async unset(editField?: EditField) {
      if (!editField) {
        editField = this.editField;
      }
      if (this.entity) {
        const opUnset: OpUnset = {
          op: OpType.Unset,
          source: OpSource.Manual,
          path: editField.path,
          system: this.systemId,
        };
        this.entity.addOpUnset(opUnset);
        this.entity.computeProperties();
        updateEntity(this.entity).subscribe(reply => {
          if (reply.error) {
            emitEditorErrorMessage(reply.error.message);
          }
        });
      }
    },
    validate(): boolean {
      if (this.entity) {
        let opSet: OpSet = {
          op: OpType.Set,
          source: OpSource.Manual,
          path: this.editField.path,
          // @ts-ignore
          value: this.currentValue,
          system: this.systemId,
        };
        let valid = this.entity.validateProp(opSet);
        if (valid.errors) {
          this.errors = valid.errors;
          return false;
        } else {
          this.errors = [];
          return true;
        }
      }
      return true;
    },
    onInputSelect(): void {
      this.onInput();
      this.onBlur();
    },
    onInput() {
      this.validate();
    },
    onFocus() {
      this.setStartValueToCurrentValue();
      this.updating = true;
    },
    onEnterKey(event: KeyboardEvent) {
      // @ts-ignore
      event.target.blur();
    },
    async onBlur(editField?: EditField, value?: unknown) {
      if (!editField || editField.type == "blur") {
        editField = this.editField;
      }
      if (!value) {
        value = this.currentValue;
      }
      this.updating = false;
      if (this.entity && !_.isEqual(this.startValue, this.currentValue)) {
        const validated = this.validate();
        if (!validated) {
          return;
        }
        let opSet: OpSet = {
          op: OpType.Set,
          source: OpSource.Manual,
          path: editField.path,
          // @ts-ignore
          value,
          system: this.systemId,
        };
        this.entity.addOpSet(opSet);
        this.entity.computeProperties();
        updateEntity(this.entity).subscribe(reply => {
          if (reply.error) {
            emitEditorErrorMessage(reply.error.message);
          }
        });
      }
    },
    setStartValueToCurrentValue() {
      this.startValue = _.cloneDeep(this.currentValue);
    },
    setCurrentValue(payload: string) {
      this.currentValue = payload;
    },
    updateOnPropChanges() {
      if (!this.updating && this.entity) {
        const startValue: string = this.entity.getProperty({
          system: this.systemId,
          path: this.editField.path,
        });
        this.setCurrentValue(_.cloneDeep(startValue));
        this.setStartValueToCurrentValue();
      }
    },
  },
  watch: {
    entity: {
      deep: true,
      immediate: true,
      handler() {
        this.updateOnPropChanges();
        this.validate();
      },
    },
  },
});
</script>
