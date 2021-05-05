<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center h-10 pt-2 pb-2 pl-6 pr-6 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">{{ entity.entityType }} {{ entity.name }} Code</div>
    </div>

    <div class="flex flex-col flex-grow w-full overflow-auto overscroll-none">
      <CodeMirror class="w-full" :value="codeValue" readOnly />
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

import CodeMirror from "@/molecules/CodeMirror.vue";
import { system$ } from "@/observables";
import { pluck, switchMap, tap } from "rxjs/operators";
import { combineLatest, Observable, of } from "rxjs";
import { SiEntity } from "si-entity";

export interface Data {
  codeValue: string;
}

export default Vue.extend({
  name: "CodeViwer",
  components: {
    CodeMirror,
  },
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
  },
  data(): Data {
    return {
      codeValue: "No code is the best code!",
    };
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
    };
  },
});
</script>
