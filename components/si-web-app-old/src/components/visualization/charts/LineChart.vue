<!-- DO NOT INCLUDE THE <TEMPLATE> TAG TO THIS VUE FILE! Vue can not merge templates. If you add an empty <template> tag, Vue will take the template from your component and not from the extended one, which will result in an empty template and errors. -->
<script>
import { Line, mixins } from "vue-chartjs";
const { reactiveProp } = mixins;

export default {
  extends: Line,
  mixins: [reactiveProp],
  props: ["options"],
  mounted() {
    this.renderChart(this.chartData, this.options);
  },
};
</script>
