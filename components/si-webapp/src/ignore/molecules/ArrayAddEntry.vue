<template>
  <div
    class="relative w-auto menu-root"
    @mouseleave="onMouseLeave"
    @mouseenter="cancelClose"
  >
    <button @click="clickButton">
      <PlusIcon size="1x" />
    </button>

    <ArrayAddEntryCategory
      :isOpen="isOpen"
      :menuItems="menuItems"
      rootMenu
      class="menu-root"
      @selected="onSelect"
    />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";
import { PlusIcon } from "vue-feather-icons";
import { EditField, OpSet, OpType, OpSource } from "si-entity/dist/siEntity";
import { EditPartial } from "si-registry/dist/registryEntry";
import { Entity } from "@/api/sdf/model/entity";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { updateEntity } from "@/observables";
import ArrayAddEntryCategory from "./ArrayAddEntry/ArrayAddEntryCategory.vue";

interface Data {
  isOpen: boolean;
}

const debounceIsOpen = _.debounce((component: any, isOpen: boolean) => {
  component.isOpen = isOpen;
}, 500);

export default Vue.extend({
  name: "ArrayAddEntry",
  components: {
    ArrayAddEntryCategory,
    PlusIcon,
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
    items: {
      type: Array as PropType<EditField[][]>,
      required: true,
    },
  },
  data(): Data {
    return {
      isOpen: false,
    };
  },
  computed: {
    menuItems(): EditPartial[] | undefined {
      if (
        this.editField.schema.type == "array" &&
        this.editField.schema.itemProperty.type == "object"
      ) {
        return this.editField.schema.itemProperty.editPartials;
      } else {
        return undefined;
      }
    },
    hasEditPartials(): boolean {
      if (this.menuItems) {
        return true;
      } else {
        return false;
      }
    },
  },
  methods: {
    clickButton(): void {
      if (this.hasEditPartials) {
        this.toggleOpen();
      } else {
        this.addItem();
      }
    },
    toggleOpen(): void {
      this.isOpen = !this.isOpen;
    },
    onSelect(name: string, event: MouseEvent): void {
      event.preventDefault();
      this.$emit("selected", name, event);
      this.isOpen = false;
      this.addItemEditPartial(name);
    },
    onMouseLeave(): void {
      if (this.menuItems) {
        debounceIsOpen(this, false);
      }
    },
    cancelClose(): void {
      if (this.menuItems) {
        debounceIsOpen.cancel();
      }
    },
    arrayEditFields(): EditField[] {
      if (this.entity) {
        let nextIndex = this.nextIndex();
        return this.entity.arrayEditFields(this.editField, nextIndex);
      } else {
        return [];
      }
    },
    nextIndex(): number {
      let fullPath = [this.entity.entityType].concat(this.editField.path);
      let arrayMetaKey = this.entity.pathToString(fullPath);
      let arrayLength = this.entity.arrayMeta[arrayMetaKey]?.length;
      if (!arrayLength) {
        arrayLength = 0;
      }
      return arrayLength;
    },
    pathRoot(): string[] {
      let path = _.cloneDeep(this.editField.path);
      const nextIndex = this.nextIndex();
      path.push(`${nextIndex}`);
      return path;
    },
    propertyPathPrefixes(
      name: string,
      pathRoot: string[],
      menuItems?: EditPartial[],
    ): string[][] | null {
      if (!menuItems) {
        menuItems = this.menuItems;
      }
      if (!menuItems) {
        return null;
      }

      for (const editPartial of menuItems) {
        if (editPartial.kind == "item") {
          if (editPartial.name == name) {
            return editPartial.propertyPaths.map(e => pathRoot.concat(e));
          }
        } else {
          const result = this.propertyPathPrefixes(
            name,
            pathRoot,
            editPartial.items,
          );
          if (result) {
            return result;
          }
        }
      }
      return null;
    },
    addItem() {
      this.$emit("add-item-edit-fields", this.arrayEditFields());
      this.setRootOp();
    },
    addItemEditPartial(name: string) {
      const entity = this.entity;
      const pathRoot = this.pathRoot();
      const pathPrefixes = this.propertyPathPrefixes(name, pathRoot);
      if (!pathPrefixes) {
        // TODO(fnichol): handle!
        throw new Error(
          "there should be property path prefixes--you messed up",
        );
      }
      const arrayEditFields = _.filter(this.arrayEditFields(), function(
        editField,
      ) {
        return _.some(pathPrefixes, function(prefix) {
          return entity.subPath(editField.path, prefix);
        });
      });
      this.$emit("add-item-edit-fields", arrayEditFields);
      this.setRootOp(name);
    },
    setRootOp(editPartial?: string) {
      const path = this.pathRoot();
      let value: unknown = "";
      if (this.editField.schema.type == "array") {
        if (this.editField.schema.itemProperty.type == "string") {
          value = "";
        } else if (this.editField.schema.itemProperty.type == "number") {
          value = 0;
        } else if (this.editField.schema.itemProperty.type == "boolean") {
          value = false;
        } else if (this.editField.schema.itemProperty.type == "object") {
          value = {};
        } else if (this.editField.schema.itemProperty.type == "array") {
          value = [];
        } else if (this.editField.schema.itemProperty.type == "map") {
          value = {};
        }
      }
      const opSet: OpSet = {
        op: OpType.Set,
        source: OpSource.Manual,
        path,
        // @ts-ignore
        value: _.cloneDeep(value),
        system: this.systemId,
      };
      if (editPartial) {
        opSet.editPartial = editPartial;
      }
      const result = this.entity.addOpSet(opSet);
      if (!result.success) {
        emitEditorErrorMessage(result.errors.join("\n"));
      }
      this.entity.computeProperties();
      updateEntity(this.entity).subscribe(reply => {
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        }
      });
    },
  },
});
</script>

<style>
.menu-root {
  z-index: 999;
}

.menu {
  background-color: #2d3748;
  border-color: #485359;
}

.menu-selected {
  background-color: #edf2f8;
  color: #000000;
}

.menu-not-selected {
  color: red;
}

.options {
  background-color: #1f2631;
  border-color: #485359;
}
.options:hover {
  background-color: #3d4b62;
  border-color: #454d3e;
}

.menu-category .category-items {
  /*  visibility: hidden; */
  position: absolute;
  left: 100%;
  top: auto;
  margin-top: -1.305rem;
}
</style>
