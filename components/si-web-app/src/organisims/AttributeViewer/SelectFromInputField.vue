<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
  >
    <template slot="widget">
      <select
        class="flex-grow pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
        placeholder="text"
        v-model="currentValue"
        :disabled="isDisabled"
        @input="onInput"
        @focus="onFocus"
        @blur="onBlur"
      >
        <option
          v-for="option in selectOptions"
          :key="option.value"
          :value="option.value"
          >{{ option.label }}
        </option>
      </select>
      <div class="flex flex-row w-10">
        <TombstoneEdit
          :entity="entity"
          :editField="editField"
          :systemId="systemId"
          @toggleTombstone="toggleTombstone"
        />
        <Unset
          :entity="entity"
          :editField="editField"
          :systemId="systemId"
          @unset="unset"
        />
      </div>
    </template>
    <template slot="value">
      {{ labelForValue }}
    </template>
  </Field>
</template>

<script lang="ts">
import _ from "lodash";

import TombstoneEdit from "@/organisims/AttributeViewer/Tombstone.vue";
import Unset from "@/organisims/AttributeViewer/Unset.vue";
import Field from "@/organisims/AttributeViewer/Field.vue";
import { ValidateFailure } from "si-entity/dist/validation";
import BaseField from "./BaseField.vue";
import { combineLatest } from "rxjs";
import { changeSet$, editSession$, workspace$ } from "@/observables";
import {
  AttributeDal,
  IGetInputLabelsRequest,
  IGetEntityRequest,
} from "@/api/sdf/dal/attributeDal";
import { SchematicKind } from "si-registry/dist/registryEntry";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { pluck, tap, switchMap } from "rxjs/operators";

interface Data {
  startValue: string;
  currentValue: string;
  updating: boolean;
  selectOptions: SelectItem[];
  errors: ValidateFailure["errors"];
  selectedOptionEntityName: string;
  selectedOptionEntityType: string;
}

interface SelectItem {
  label: string;
  value: string | number;
}

export default BaseField.extend({
  name: "SelectFromField",
  mixins: [BaseField],
  components: {
    TombstoneEdit,
    Unset,
    Field,
  },
  data(): Data {
    return {
      startValue: "",
      currentValue: "",
      updating: false,
      errors: [],
      selectOptions: [],
      selectedOptionEntityName: "unknown",
      selectedOptionEntityType: "unknown",
    };
  },
  subscriptions: function(this: any): Record<string, any> {
    let entity$ = this.$watchAsObservable("entity", { immediate: true }).pipe(
      pluck("newValue"),
    );
    let getSelectOptions$ = combineLatest(
      workspace$,
      changeSet$,
      editSession$,
      entity$,
    ).pipe(
      tap(async ([workspace, changeSet, editSession, entity]) => {
        if (
          this.editField.schema.widget?.name == "selectFromInput" &&
          workspace
        ) {
          let request: IGetInputLabelsRequest = {
            workspaceId: workspace.id,
            // @ts-ignore
            entityId: entity.id,
            inputName: this.editField.schema.widget?.inputName,
            schematicKind: SchematicKind.Component,
          };
          if (changeSet) {
            request.changeSetId = changeSet.id;
          }
          if (editSession) {
            request.editSessionId = editSession.id;
          }
          let reply = await AttributeDal.getInputLabels(request);
          if (reply.error) {
            if (reply.error.code != 406) {
              emitEditorErrorMessage(reply.error.message);
            }
            this.selectOptions = [];
            return;
          } else {
            this.selectOptions = reply.items;
            return;
          }
        } else {
          this.selectOptions = [];
          return [];
        }
      }),
    );
    return {
      changeSet: changeSet$,
      editSession: editSession$,
      workspace: workspace$,
      getSelectOptions: getSelectOptions$,
    };
  },
  computed: {
    labelForValue(): string | undefined {
      let selectedItem = _.find(this.selectOptions, [
        "value",
        this.currentValue,
      ]);
      if (selectedItem) {
        return selectedItem["label"];
      } else {
        return undefined;
      }
    },
  },
});
</script>
