<template>
  <div class="flex flex-row items-center w-full mt-2">
    <div class="flex flex-col w-full">
      <div class="flex flex-row items-center w-full">
        <div class="w-40 px-2 text-sm leading-tight text-right text-white">
          {{ fieldName }}
        </div>
        <div
          class="flex flex-grow pl-2 mr-2 mr-10 text-sm leading-tight text-gray-400"
          v-if="editMode"
          @keyup.stop
          @keydown.stop
        >
          <input
            class="flex-grow pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property disabled:opacity-50"
            type="text"
            aria-label="name"
            placeholder="text"
            v-model="currentValue"
            :disabled="isDisabled"
            @input="onInput"
            @focus="onFocus"
            @blur="onBlur"
          />
          <!-- TODO: Fix up the icons, set the colors, expand to see the values, refactor  -->
          <div class="flex items-center w-10 ml-1">
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
                    <div
                      class="flex flex-row items-center w-full text-sm text-white"
                    >
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
                                <button
                                  @click="toggleTombstone({ source: 'manual' })"
                                >
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
                            <div
                              class="flex flex-row pt-1 border-t-2 border-gray-600"
                            >
                              <div
                                class="flex"
                                :class="sourceClasses({ source: 'inferred' })"
                              >
                                <button
                                  @click="
                                    toggleTombstone({ source: 'inferred' })
                                  "
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
                            <button
                              @click="toggleTombstone({ source: 'manual' })"
                            >
                              <CircleSystemIcon size="1x" />
                            </button>
                          </div>
                          <div
                            class="ml-2"
                            :class="sourceClasses({ source: 'inferred' })"
                          >
                            <button
                              @click="toggleTombstone({ source: 'inferred' })"
                            >
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
                            <div
                              class="flex flex-row pt-1 border-t-2 border-gray-600"
                            >
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
                              <div class="flex flex-grow ml-2">
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
            <button @click="unset">
              <Trash2Icon size="1x" v-if="hasManualValue" class="ml-1" />
            </button>
          </div>
        </div>
        <div
          v-else
          class="flex flex-grow pl-2 mr-2 text-sm leading-tight text-gray-400"
        >
          <template v-if="entity">
            {{ entity.name }}
          </template>
        </div>
      </div>
      <div class="flex flex-row w-full">
        <div class="w-40"></div>
        <div class="flex flex-grow pl-2 mr-10">
          <ValidationErrors :errors="errors" />
        </div>
      </div>
    </div>
  </div>
</template>

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

import ValidationErrors from "@/organisims/AttributeViewer/ValidationErrors.vue";
import Tooltip from "@/atoms/Tooltip.vue";
import { Trash2Icon, CircleIcon, MoreHorizontalIcon } from "vue-feather-icons";
import CircleApplicationIcon from "./icons/CircleApplicationIcon.vue";
import CircleSystemIcon from "./icons/CircleSystemIcon.vue";
import SquareApplicationIcon from "./icons/SquareApplicationIcon.vue";
import SquareSystemIcon from "./icons/SquareSystemIcon.vue";
import { updateEntity } from "@/observables";
import { Entity } from "@/api/sdf/model/entity";

interface Data {
  startValue: string;
  currentValue: string;
  updating: boolean;
  errors: string[];
  showAllValues: boolean;
}

export default Vue.extend({
  name: "TextField",
  components: {
    ValidationErrors,
    Trash2Icon,
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
      startValue: "",
      currentValue: "",
      updating: false,
      errors: [],
      showAllValues: false,
    };
  },
  computed: {
    fieldName(): string {
      if (this.editField) {
        return this.editField.name;
      } else {
        return "schema bug!";
      }
    },
    hasManualValue(): boolean {
      if (this.entity) {
        return this.entity.hasValueFrom({
          path: this.editField.path,
          source: OpSource.Manual,
          system: this.systemId,
        });
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
    async toggleTombstone({
      source,
      system,
    }: {
      source: OpTombstone["source"];
      system?: OpTombstone["system"];
    }) {
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
            path: this.editField.path,
            system: systemId,
          })
        ) {
          const opTombstone: OpTombstone = {
            op: OpType.Tombstone,
            source,
            path: this.editField.path,
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
            path: this.editField.path,
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
    async unset() {
      if (this.entity) {
        const opUnset: OpUnset = {
          op: OpType.Unset,
          source: OpSource.Manual,
          path: this.editField.path,
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
      if (this.entity && this.startValue != this.currentValue) {
        let opSet: OpSet = {
          op: OpType.Set,
          source: OpSource.Manual,
          path: this.editField.path,
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
    onInput() {
      this.validate();
    },
    onFocus() {
      this.setStartValueToCurrentValue();
      this.updating = true;
    },
    async onBlur() {
      this.updating = false;
      if (this.entity && this.startValue != this.currentValue) {
        const validated = this.validate();
        if (!validated) {
          return;
        }
        let opSet: OpSet = {
          op: OpType.Set,
          source: OpSource.Manual,
          path: this.editField.path,
          value: this.currentValue,
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
      },
    },
  },
});
</script>
