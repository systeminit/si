<template>
  <Field
    :name="fieldName"
    :showField="showField"
    :errors="errors"
    :editMode="editMode"
    :nameClasses="fieldNameColor"
  >
    <template slot="widget">
      <select
        class="flex-grow pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
        :class="borderColor"
        placeholder="text"
        v-model="currentValue"
        :disabled="isDisabled"
        @change="onInputSelect"
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
      <span :class="textColor"> {{ labelForValue }} </span>
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
import {
  changeSet$,
  editSession$,
  workspace$,
  refreshSecretList$,
} from "@/observables";
import {
  AttributeDal,
  IGetInputLabelsRequest,
  IGetEntityRequest,
} from "@/api/sdf/dal/attributeDal";
import { SchematicKind } from "si-registry/dist/registryEntry";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { pluck, tap, switchMap } from "rxjs/operators";
import { IListSecretsRequest, SecretDal } from "@/api/sdf/dal/secretDal";

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
  name: "SelectFromSecretField",
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
      entity$,
      refreshSecretList$,
    ).pipe(
      tap(async ([workspace, entity]) => {
        if (
          this.editField.schema.widget?.name == "selectFromSecret" &&
          workspace
        ) {
          let request: IListSecretsRequest = {
            workspaceId: workspace.id,
          };
          let reply = await SecretDal.listSecrets(request);
          if (reply.error) {
            if (reply.error.code != 406) {
              emitEditorErrorMessage(reply.error.message);
            }
            this.selectOptions = [];
            return;
          } else {
            let filteredList = _.filter(reply.list, secret => {
              return this.editField.schema.widget.secretKind == secret.kind;
            });
            let labels = _.map(filteredList, secret => {
              return { value: secret.id, label: secret.name };
            });
            this.selectOptions = labels;
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
