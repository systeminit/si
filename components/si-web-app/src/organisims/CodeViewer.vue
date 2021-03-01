<template>
  <div id="code-viewer" class="flex flex-col w-full h-full">
    <div class="flex flex-row justify-end h-6 bg-black">
      <div class="flex justify-start flex-grow" v-if="currentObject">
        {{ currentObject.name }} ({{ currentObject.objectType }})
      </div>
      <div class="pr-2">
        <SiButton
          label="discard"
          kind="cancel"
          icon="cancel"
          size="xs"
          @click.native="discardEdits"
          :disabled="!hasChanges"
        />
      </div>
      <div class="pr-2">
        <SiButton
          label="apply"
          kind="save"
          icon="save"
          size="xs"
          :disabled="!hasChanges"
          @click.native="saveDifferences"
        />
      </div>
    </div>
    <div class="w-full" style="height: 90%">
      <textarea id="codemirror-mount" class="w-full"> </textarea>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { mapState, mapGetters } from "vuex";
import codemirror from "codemirror";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/base16-dark.css";
import "codemirror/mode/yaml/yaml";
import YAML from "yaml";

import { Node } from "@/api/sdf/model/node";
import _ from "lodash";
import { InstanceStoreContext } from "@/store";
import { AttributeStore } from "@/store/modules/attribute";
import { Entity } from "@/api/sdf/model/entity";
import { RegistryProperty, registryProperty } from "@/api/registryProperty";
import { diffEntity, DiffResult } from "@/api/diff";
import {
  emitPropMapAddEvent,
  emitPropRepeatedAddEvent,
  IPropChangeEvent,
  offPropChangeEvent,
  onPropChangeEvent,
} from "@/atoms/PanelEventBus";
import SiButton from "@/atoms/SiButton.vue";
import { SessionStore } from "@/store/modules/session";
import { EditorStore } from "@/store/modules/editor";
import { IEntitySetPropertyRequest } from "@/api/sdf/dal/editorDal";

// We want to take the document off the code property as your starting point.
// Then when you edit it, we take it in, parse it into JSON, then run the
// diff algo against it. If it has new entries, then we make the set ops for
// them! It's going to be epic.
// Also, mark the lines with the gutter if we can.
interface IData {
  codemirror: null | CodeMirror.Editor;
  currentValue: string | null;
  baseValue: string | null;
  entryValue: string | null;
}

export default Vue.extend({
  name: "CodeViewer",
  props: {
    attributeStoreCtx: {
      type: Object as PropType<InstanceStoreContext<AttributeStore>>,
    },
  },
  components: {
    SiButton,
  },
  data(): IData {
    return {
      codemirror: null,
      currentValue: null,
      baseValue: null,
      entryValue: null,
    };
  },
  computed: {
    hasChanges(): boolean {
      if (this.codemirror && this.entryValue) {
        return this.entryValue != this.codemirror.getValue();
      } else {
        return false;
      }
    },
    editMode(): boolean {
      return this.$store.getters["editor/inEditable"];
    },
    currentObject(): Entity | null {
      return this.attributeStoreCtx.state.currentObject;
    },
    baseObject(): Entity | null {
      return this.attributeStoreCtx.state.baseObject;
    },
    codeProperty(): RegistryProperty | null {
      if (this.currentObject) {
        let propertyList = registryProperty.propertyList(this.currentObject);
        let codeProperty = _.find(propertyList, ["kind", "code"]);
        if (codeProperty) {
          return codeProperty;
        } else {
          return null;
        }
      } else {
        return null;
      }
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
  },
  methods: {
    changedElement(): HTMLElement {
      let marker = document.createElement("div");
      marker.classList.add("text-gold");
      marker.innerHTML = ">>";
      return marker;
    },
    updateModel(): void {
      if (this.codemirror) {
        if (this.currentValue) {
          this.codemirror.setValue(this.currentValue);
          if (!this.editMode) {
            this.codemirror.setOption("readOnly", true);
          }
          if (this.currentObject && this.baseObject) {
            let ydoc = YAML.parseDocument(this.currentValue);
            let diffResult = diffEntity(this.baseObject, this.currentObject);
            let codeProperty = this.codeProperty;
            for (let diffEntry of diffResult.entries) {
              if (
                _.isEqual(diffEntry.path.slice(0, 3), [
                  "properties",
                  "__baseline",
                  "kubernetesObject", // TODO: refactor so its easy to extract the pair of a code property
                ])
              ) {
                let changePath = diffEntry.path.slice(3, diffEntry.path.length);
                // @ts-ignore
                let yamlItems = ydoc.contents?.items;
                PATHWALK: for (let x = 0; x < changePath.length; x++) {
                  const pathItem = changePath[x];
                  let match = _.find(yamlItems, ["key.value", pathItem]);
                  if (!match) {
                    let pathItemNumber = _.toNumber(pathItem);
                    if (pathItemNumber != NaN) {
                      match = yamlItems[pathItemNumber];
                    }
                  }
                  if (match) {
                    if (x == changePath.length - 1) {
                      let startPosition = match.key.range[0];
                      let yamlToStartPosition = this.currentValue.slice(
                        0,
                        startPosition,
                      );
                      let lineNumber =
                        yamlToStartPosition.split(/\n/).length - 1;
                      this.codemirror.setGutterMarker(
                        lineNumber,
                        "edited",
                        this.changedElement(),
                      );
                    } else {
                      if (match.value?.items) {
                        yamlItems = match.value.items;
                      } else {
                        yamlItems = match.items;
                      }
                    }
                  } else {
                    continue PATHWALK;
                  }
                }
              }
            }
          }
        } else {
          this.codemirror.setValue("# No code property!");
          this.codemirror.setOption("readOnly", true);
        }
        this.codemirror.refresh();
      }
    },
    onPropChange(event: IPropChangeEvent) {
      this.setValuesFromObjects();
      this.updateModel();
    },
    setValuesFromObjects(): void {
      if (this.codeProperty && this.currentObject && this.baseObject) {
        this.currentValue = _.cloneDeep(
          _.get(this.currentObject, this.codeProperty.path),
        );
        this.baseValue = _.cloneDeep(
          _.get(this.baseObject, this.codeProperty.path),
        );
        this.entryValue = this.currentValue;
      }
    },
    async discardEdits() {
      if (this.codemirror && this.entryValue) {
        this.currentValue = this.entryValue;
        this.updateModel();
      }
    },
    async saveDifferences() {
      if (this.codemirror) {
        let editorValue = this.codemirror.getValue();
        if (this.entryValue && editorValue) {
          if (this.entryValue == editorValue) {
            console.log("value has not changed", this.entryValue, editorValue);
            return;
          }
          let fakeCurrentObject = _.cloneDeep(this.currentObject);
          let fakeEntryObject = _.cloneDeep(this.currentObject);
          let codeProperty = this.codeProperty;
          if (
            fakeCurrentObject &&
            fakeEntryObject &&
            codeProperty &&
            this.codemirror
          ) {
            let path = codeProperty.path.slice(0, codeProperty.path.length - 1);
            path.push("kubernetesObject");
            _.set(fakeCurrentObject, path, YAML.parse(editorValue));
            _.set(fakeEntryObject, path, YAML.parse(this.entryValue));

            if (
              this.currentWorkspace &&
              this.currentChangeSet &&
              this.currentEditSession &&
              this.currentObject
            ) {
              let diffResults = diffEntity(fakeEntryObject, fakeCurrentObject);
              let baseRequest = {
                workspaceId: this.currentWorkspace.id,
                entityId: fakeCurrentObject.id,
                changeSetId: this.currentChangeSet.id,
                editSessionId: this.currentEditSession.id,
              };
              console.log("diff results", diffResults);
              let bulkProperties = [];
              for (const diffEntry of diffResults.entries) {
                if (diffEntry.kind == "edit" || diffEntry.kind == "add") {
                  let path = diffEntry.path;
                  bulkProperties.push({ path, value: diffEntry.after });
                }
              }
              await this.$store.dispatch("editor/entitySetPropertyBulk", {
                ...baseRequest,
                properties: bulkProperties,
              });
              for (const diffEntry of diffResults.entries) {
                if (diffEntry.kind == "add") {
                  let path = diffEntry.path;
                  let propertyList = registryProperty.propertyList(
                    this.currentObject,
                  );
                  let diffProperty = _.find(propertyList, ["path", path]);
                  if (_.isUndefined(diffProperty)) {
                    diffProperty = _.find(propertyList, [
                      "path",
                      path.slice(0, path.length - 1),
                    ]);
                  }
                  if (diffProperty && diffProperty.kind == "map") {
                    emitPropMapAddEvent(
                      { path },
                      this.currentObject.id,
                      "add",
                      diffEntry.after,
                    );
                  } else {
                    for (let x = 0; x < path.length - 1; x++) {
                      let pathToCheck = path.slice(0, path.length - x);
                      let diffProperty = _.find(propertyList, [
                        "path",
                        pathToCheck,
                      ]);
                      if (diffProperty && diffProperty.repeated) {
                        let fakeCurrentObjectValue = _.get(
                          fakeCurrentObject,
                          pathToCheck,
                        );
                        let fakeEntryObjectValue = _.get(
                          fakeEntryObject,
                          pathToCheck,
                        );
                        if (
                          fakeCurrentObjectValue.length >
                          fakeEntryObjectValue.length
                        ) {
                          emitPropRepeatedAddEvent(
                            { path: pathToCheck },
                            this.currentObject.id,
                            "add",
                            {
                              from: this.attributeStoreCtx.dispatchPath(),
                            },
                          );
                        }
                      }
                    }
                    console.log("who the fuck knows", {
                      diffProperty,
                      path,
                      diffEntry,
                    });
                  }
                } else if (diffEntry.kind == "delete") {
                  console.log("go a delete", { diffEntry });
                }
              }
            }
          }
          this.entryValue = editorValue;
        }
      }
    },
    saveEntryValue() {
      this.entryValue = this.currentValue;
    },
  },
  async mounted() {
    this.setValuesFromObjects();
    const mountPoint = document.getElementById(
      "codemirror-mount",
    ) as HTMLTextAreaElement;
    if (mountPoint) {
      const doc = codemirror.fromTextArea(mountPoint, {
        lineNumbers: true,
        viewportMargin: Infinity,
        mode: "yaml",
        theme: "base16-dark",
        screenReaderLabel: "code-mirror-editor",
        gutters: ["CodeMirror-linenumbers", "edited"],
      });
      doc.setSize("100%", "100%");
      const codeViewerComponent = this;
      doc.on("focus", function() {
        codeViewerComponent.saveEntryValue();
      });
      //doc.on("blur", function() {
      //  let currentValue = doc.getValue();
      //  if (currentValue != "# No Code!") {
      //    codeViewerComponent.fieldValue = currentValue;
      //  }
      //});

      this.codemirror = doc;

      this.updateModel();
    }
  },
  created() {
    if (this.codeProperty && this.currentObject) {
      onPropChangeEvent(
        this.codeProperty,
        this.currentObject.id,
        this.onPropChange,
      );
    }
  },
  beforeDestroy() {
    if (this.codeProperty && this.currentObject) {
      offPropChangeEvent(
        this.codeProperty,
        this.currentObject.id,
        this.onPropChange,
      );
    }
  },
  watch: {
    currentObject(): void {
      this.currentValue = null;
      this.entryValue = null;
      this.setValuesFromObjects();
      this.updateModel();
    },
  },
});
</script>

<style>
.CodeMirror-gutters {
  border-right: 1px solid #333 !important;
  padding-right: 2em !important;
  white-space: nowrap !important;
}
.CodeMirror-gutter-elt {
  min-width: 10px !important;
}
</style>
