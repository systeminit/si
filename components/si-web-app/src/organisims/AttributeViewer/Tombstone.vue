<template>
  <div class="flex items-center ml-1">
    <button>
      <Tooltip sticky alignRight onlyOnClick :offset="20">
        <div
          v-if="
            valueOpTypeForLayer({
              system: systemId,
              path: editField.path,
              layer: 'systemManual',
            })
          "
          :class="valueFromClasses"
        >
          <CircleSystemIcon size="1x" />
        </div>
        <div
          v-else-if="
            valueOpTypeForLayer({
              system: systemId,
              path: editField.path,
              layer: 'systemInferred',
            })
          "
          :class="valueFromClasses"
        >
          <SquareSystemIcon size="1x" />
        </div>
        <div
          v-else-if="
            valueOpTypeForLayer({
              system: systemId,
              path: editField.path,
              layer: 'applicationManual',
            })
          "
          :class="valueFromClasses"
        >
          <CircleApplicationIcon size="1x" />
        </div>
        <div
          v-else-if="
            valueOpTypeForLayer({
              system: systemId,
              path: editField.path,
              layer: 'applicationInferred',
            })
          "
          :class="valueFromClasses"
        >
          <SquareApplicationIcon size="1x" />
        </div>
        <div :class="valueFromClasses" v-else>
          <CircleIcon size="1x" />
        </div>
        <template v-slot:tooltip>
          <div class="flex flex-col w-52">
            <div class="flex flex-row items-center w-full text-sm text-white">
              <div class="flex">
                Sources
              </div>
            </div>
            <div class="flex flex-row w-full">
              <div class="w-20 ml-1 text-right">
                System:
              </div>
              <div class="flex flex-col flex-grow ml-2">
                <div class="flex flex-row w-full" v-if="showAllValues">
                  <div class="flex flex-col w-full">
                    <div class="flex flex-row mb-1">
                      <div
                        class="flex"
                        :class="sourceClasses({ source: 'manual' })"
                      >
                        <button @click="toggleTombstone({ source: 'manual' })">
                          <CircleSystemIcon size="1x" />
                        </button>
                      </div>
                      <div class="flex flex-grow ml-2">
                        {{
                          valueFrom({
                            source: "manual",
                            system: systemId,
                          })
                        }}
                      </div>
                    </div>
                    <div class="flex flex-row pt-1 border-t-2 border-gray-600">
                      <div
                        class="flex"
                        :class="sourceClasses({ source: 'inferred' })"
                      >
                        <button
                          @click="toggleTombstone({ source: 'inferred' })"
                        >
                          <SquareSystemIcon size="1x" />
                        </button>
                      </div>
                      <div class="flex flex-grow ml-2">
                        {{
                          valueFrom({
                            source: "inferred",
                            system: systemId,
                          })
                        }}
                      </div>
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full" v-else>
                  <div :class="sourceClasses({ source: 'manual' })">
                    <button @click="toggleTombstone({ source: 'manual' })">
                      <CircleSystemIcon size="1x" />
                    </button>
                  </div>
                  <div
                    class="ml-2"
                    :class="sourceClasses({ source: 'inferred' })"
                  >
                    <button @click="toggleTombstone({ source: 'inferred' })">
                      <SquareSystemIcon size="1x" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
            <div class="flex flex-row w-full">
              <div class="w-20 ml-1 text-right">
                Application:
              </div>
              <div class="flex flex-col flex-grow ml-2">
                <div class="flex flex-row w-full" v-if="showAllValues">
                  <div class="flex flex-col w-full">
                    <div
                      class="flex flex-row pt-1 mt-1 mb-1 border-t-2 border-gray-600"
                    >
                      <div
                        class="flex"
                        :class="
                          sourceClasses({
                            source: 'manual',
                            system: 'baseline',
                          })
                        "
                      >
                        <button
                          @click="
                            toggleTombstone({
                              source: 'manual',
                              system: 'baseline',
                            })
                          "
                        >
                          <CircleApplicationIcon size="1x" />
                        </button>
                      </div>
                      <div class="flex flex-grow ml-2">
                        {{
                          valueFrom({
                            source: "manual",
                            system: "baseline",
                          })
                        }}
                      </div>
                    </div>
                    <div class="flex flex-row pt-1 border-t-2 border-gray-600">
                      <div
                        class="flex"
                        :class="
                          sourceClasses({
                            source: 'inferred',
                            system: 'baseline',
                          })
                        "
                      >
                        <button
                          @click="
                            toggleTombstone({
                              source: 'inferred',
                              system: 'baseline',
                            })
                          "
                        >
                          <SquareApplicationIcon size="1x" />
                        </button>
                      </div>
                      <div class="flex  flex-grow ml-2">
                        {{
                          valueFrom({
                            source: "inferred",
                            system: "baseline",
                          })
                        }}
                      </div>
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full" v-else>
                  <div
                    :class="
                      sourceClasses({
                        source: 'manual',
                        system: 'baseline',
                      })
                    "
                  >
                    <button
                      @click="
                        toggleTombstone({
                          source: 'manual',
                          system: 'baseline',
                        })
                      "
                    >
                      <CircleApplicationIcon size="1x" />
                    </button>
                  </div>
                  <div
                    class="ml-2"
                    :class="
                      sourceClasses({
                        source: 'inferred',
                        system: 'baseline',
                      })
                    "
                  >
                    <button
                      @click="
                        toggleTombstone({
                          source: 'inferred',
                          system: 'baseline',
                        })
                      "
                    >
                      <SquareApplicationIcon size="1x" />
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <div class="flex items-center justify-end flex-grow">
              <button @click="toggleShowAllValues">
                <MoreHorizontalIcon size="1x" />
              </button>
            </div>
          </div>
        </template>
      </Tooltip>
    </button>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Entity } from "@/api/sdf/model/entity";
import {
  EditField,
  OpSet,
  OpType,
  OpSource,
  OpTombstone,
} from "si-entity/dist/siEntity";
import { updateEntity } from "@/observables";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import Tooltip from "@/atoms/Tooltip.vue";
import { Trash2Icon, CircleIcon, MoreHorizontalIcon } from "vue-feather-icons";
import CircleApplicationIcon from "./icons/CircleApplicationIcon.vue";
import CircleSystemIcon from "./icons/CircleSystemIcon.vue";
import SquareApplicationIcon from "./icons/SquareApplicationIcon.vue";
import SquareSystemIcon from "./icons/SquareSystemIcon.vue";

import _ from "lodash";

interface Data {
  showAllValues: boolean;
}

export default Vue.extend({
  name: "Tombstone",
  components: {
    CircleIcon,
    Tooltip,
    CircleApplicationIcon,
    CircleSystemIcon,
    SquareApplicationIcon,
    SquareSystemIcon,
    MoreHorizontalIcon,
  },
  props: {
    entity: {
      type: Object as PropType<Entity>,
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
      showAllValues: false,
    };
  },
  methods: {
    valueFrom({
      source,
      system,
    }: {
      source: OpSet["source"];
      system: OpSet["system"];
    }): string | number | boolean | undefined {
      if (this.entity) {
        let result = this.entity.valueFrom({
          source,
          system,
          path: this.editField.path,
        });
        if (_.isUndefined(result)) {
          return "---";
        } else {
          return result;
        }
      }
    },
    toggleShowAllValues() {
      if (this.showAllValues) {
        this.showAllValues = false;
      } else {
        this.showAllValues = true;
      }
    },
    valueOpTypeForLayer({
      path,
      system,
      layer,
    }: {
      path: OpSet["path"];
      system: OpSet["system"];
      layer:
        | "systemManual"
        | "systemInferred"
        | "applicationManual"
        | "applicationInferred";
    }): boolean {
      if (this.entity) {
        const op = this.entity.valueOpForPath({ path, system });
        if (op) {
          if (layer == "systemManual") {
            return op.system == this.systemId && op.source == OpSource.Manual;
          } else if (layer == "systemInferred") {
            return op.system == this.systemId && op.source == OpSource.Inferred;
          } else if (layer == "applicationManual") {
            return op.system == "baseline" && op.source == OpSource.Manual;
          } else if (layer == "applicationInferred") {
            return op.system == "baseline" && op.source == OpSource.Inferred;
          }
        }
      }
      return false;
    },
    toggleTombstone({
      source,
      system,
    }: {
      source: OpTombstone["source"];
      system?: OpTombstone["system"];
    }) {
      this.$emit("toggleTombstone", { source, system });
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
    valueFromClasses(): Record<string, any> {
      if (this.entity) {
        if (this.entity.isPathTombstoned(this.editField.path)) {
          return { "text-red-100": true };
        }
      }
      return {};
    },
    sourceClasses({
      source,
      system,
    }: {
      source: OpTombstone["source"];
      system?: OpTombstone["system"];
    }): Record<string, boolean> {
      let classes: Record<string, boolean> = {};
      let systemId;
      if (system) {
        systemId = system;
      } else {
        systemId = this.systemId;
      }
      if (
        this.hasTombstone({
          source,
          path: this.editField.path,
          system: systemId,
        })
      ) {
        classes["text-red-300"] = true;
      } else {
        classes["text-blue-300"] = true;
      }
      return classes;
    },
  },
});
</script>
