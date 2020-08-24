import Vue from "vue";
import { mapGetters } from "vuex";
import { RegistryProperty } from "@/store/modules/node";
import _ from "lodash";

interface Data {
  originalValue: any;
}

export default Vue.extend({
  props: {
    entityProperty: Object as () => RegistryProperty,
  },
  data() {
    let fieldValue = _.cloneDeep(
      this.$store.getters["node/getFieldValue"](this.entityProperty.path),
    );
    if (this.entityProperty.prop.kind() == "map") {
      if (fieldValue == undefined) {
        fieldValue = [];
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
      let value = this.fieldValue;
      let map = this.entityProperty.prop.kind() == "map";
      if (
        this.entityProperty.prop.kind() == "number" &&
        // @ts-ignore - we know you don't have it everywhere
        this.entityProperty.prop.numberKind == "int32"
      ) {
        value = parseInt(value);
      }
      if (!_.isEqual(this.originalValue, this.fieldValue)) {
        await this.$store.dispatch("node/setFieldValue", {
          path: this.entityProperty.path,
          value,
          map,
        });
      }
    },
  },
  computed: {
    ...mapGetters({
      diff: "node/diffCurrent",
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
        return _.isEqual(diffEntry.path, this.entityProperty.path);
      });
      if (result) {
        return true;
      } else {
        return false;
      }
    },
  },
  watch: {
    entityProperty() {
      let fieldValue = this.$store.getters["node/getFieldValue"](
        this.entityProperty.path,
      );
      if (this.entityProperty.prop.kind() == "map") {
        if (fieldValue == undefined) {
          fieldValue = [];
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
});
