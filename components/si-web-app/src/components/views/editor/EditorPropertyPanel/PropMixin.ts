import Vue from "vue";
import { mapGetters, mapState } from "vuex";
import { RegistryProperty } from "@/api/sdf/model/node";
import _ from "lodash";

interface Data {
  fieldValue: any;
  originalValue: any;
}

export default Vue.extend({
  props: {
    entityProperty: Object as () => RegistryProperty,
  },
  data(): Data {
    let fieldValue: any = _.get(
      this.$store.state.editor.editObject.properties["__baseline"],
      this.entityProperty.path,
    );
    if (this.entityProperty.kind == "map") {
      if (fieldValue == undefined) {
        fieldValue = {};
      }
    } else if (this.entityProperty.prop.repeated) {
      if (!fieldValue) {
        fieldValue = [];
      }
    }

    return {
      fieldValue: _.cloneDeep(fieldValue),
      originalValue: _.cloneDeep(fieldValue),
    };
  },
  methods: {
    storeStartingValue(): void {
      this.originalValue = this.fieldValue;
    },
    async saveIfModified(): Promise<void> {
      if (!_.isEqual(this.originalValue, this.fieldValue)) {
        if (this.entityProperty.kind == "number") {
          let fieldValueAsNumber = parseInt(this.fieldValue, 10);
          await this.$store.dispatch("editor/entitySet", {
            path: this.entityProperty.path,
            value: fieldValueAsNumber,
          });
        } else {
          await this.$store.dispatch("editor/entitySet", {
            path: this.entityProperty.path,
            value: this.fieldValue,
          });
        }
        if (this.entityProperty.prop.sync) {
          await this.$store.dispatch("editor/syncCurrentResource");
        }
      }
    },
  },
  watch: {
    editObject() {
      let fieldValue: any = _.get(
        this.editObject.properties["__baseline"],
        this.entityProperty.path,
      );
      if (this.entityProperty.kind == "map") {
        if (fieldValue == undefined) {
          fieldValue = {};
        }
      } else if (this.entityProperty.prop.repeated) {
        if (!fieldValue) {
          fieldValue = [];
        }
      }

      this.fieldValue = _.cloneDeep(fieldValue);
      this.originalValue = _.cloneDeep(fieldValue);
    },
  },
  computed: {
    ...mapState({
      editObject: (state: any): any => state.editor.editObject,
      diff: (state: any): any => state.editor.diff,
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
      let result = _.find(this.diff.entries, diffEntry => {
        return _.isEqual(
          diffEntry.path,
          ["properties", "__baseline"].concat(this.entityProperty.path),
        );
      });
      if (result) {
        return true;
      } else {
        return false;
      }
    },
  },
});
