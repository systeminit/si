<template>
  <div>
    <input
      type="text"
      list="instanceTypes"
      :readonly="readonly"
      :value="value"
      @input="change($event)"
      @dblclick.stop
      @pointerdown.stop
      @pointermove.stop
    />
    <datalist id="instanceTypes">
      <option value="c5.large"></option>
      <option value="c5.xlarge"></option>
      <option value="r5.large"></option>
      <option value="r5.xlarge"></option>
    </datalist>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "InstanceTypeControl",
  props: ["readonly", "emitter", "ikey", "getData", "putData"],
  data() {
    return {
      value: "test",
    };
  },
  methods: {
    // @ts-ignore: Parameter 'e' implicitly has an 'any' type.
    change(e) {
      this.value = e.target.value;
      this.update();
    },
    update() {
      if (this.ikey) this.putData(this.ikey, this.value);
      this.emitter.trigger("process");
    },
  },
  mounted() {
    this.value = this.getData(this.ikey);
  },
});
</script>
<style>
html,
body {
  height: 100%;
  width: 100%;
}

.node .control input,
.node .input-control input {
  width: 140px;
}
select,
input {
  width: 100%;
  border-radius: 30px;
  background-color: white;
  padding: 2px 6px;
  border: 1px solid #999;
  font-size: 110%;
  width: 170px;
}
</style>
