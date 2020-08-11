import Vue from "vue";
import { mapGetters } from "vuex";
import { RegistryProperty } from "@/store/modules/node";
import _ from "lodash";

export default Vue.extend({
  props: {
    entityProperty: Object as () => RegistryProperty,
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
});
