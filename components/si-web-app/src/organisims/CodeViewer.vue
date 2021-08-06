<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">{{ entity.entityType }} {{ entity.name }} Code</div>
    </div>

    <div class="flex flex-col flex-grow w-full overflow-auto overscroll-none">
      <Monaco
        class="w-full h-full"
        :readOnly="!editMode"
        :value="codeValue"
        @input="setNewCodeValue"
        @blur="updateEntityFromCode"
      />
    </div>
  </div>
</template>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>

<script lang="ts">
import Vue, { PropType } from "vue";

import { Entity } from "@/api/sdf/model/entity";

import Monaco from "@/molecules/Monaco.vue";
import { system$, updateEntity, editMode$ } from "@/observables";
import { pluck, switchMap, tap } from "rxjs/operators";
import { combineLatest, Observable, of } from "rxjs";
import { Diff } from "@/api/sdf/model/diff";
import { SiEntity, OpSource } from "si-entity";
import { Qualification } from "@/api/sdf/model/qualification";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import _ from "lodash";

export interface Data {
  codeValue: string;
  newCodeValue: string;
}

export default Vue.extend({
  name: "CodeViewer",
  components: {
    Monaco,
  },
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    diff: {
      type: Array as PropType<Diff>,
      required: true,
    },
    qualifications: {
      type: Array as PropType<Qualification[]>,
    },
  },
  data(): Data {
    return {
      codeValue: "No code is the best code!",
      newCodeValue: "No code is the best code!",
    };
  },
  methods: {
    setNewCodeValue(value: string, _event: any) {
      this.newCodeValue = value;
    },
    updateEntityFromCode() {
      const code = this.newCodeValue;
      if (
        _.isEqual(this.newCodeValue, this.codeValue) ||
        this.newCodeValue == "No code is the best code!"
      ) {
        return;
      }
      console.log("hrm", {
        codeValue: this.codeValue,
        newCodeValue: this.newCodeValue,
      });

      // @ts-ignore
      if (this.system) {
        // @ts-ignore
        this.entity.setCode(OpSource.Manual, this.system.id, code);
        this.entity.computeProperties();
        updateEntity(this.entity).subscribe(reply => {
          if (reply.error) {
            emitEditorErrorMessage(reply.error.message);
          }
        });
      }
    },
  },
  subscriptions: function(this: any): Record<string, any> {
    let entity$: Observable<Entity> = this.$watchAsObservable("entity", {
      immediate: true,
    }).pipe(pluck("newValue"));
    let codeValueLatest$ = combineLatest(entity$, system$).pipe(
      switchMap(([ientity, system]) => {
        let entity = SiEntity.fromJson(ientity);
        //entity.computeCode();
        if (system) {
          return of(entity.getCode(system.id));
        } else {
          return of(entity.getCode("baseline"));
        }
      }),
      tap(code => {
        if (code) {
          this.codeValue = code;
        } else {
          this.codeValue = "No code is the best code!";
        }
      }),
    );
    return {
      system: system$,
      codeValueLatest: codeValueLatest$,
      editMode: editMode$,
    };
  },
});
</script>
