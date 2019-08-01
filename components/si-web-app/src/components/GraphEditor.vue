<template>
  <div id="rete" ref="rete">
  </div>
</template>

<script>
import Vue from "vue";

// import Alight from "alight";

import Rete from "rete";
// import Component from "rete";

import ConnectionPlugin from "rete-connection-plugin";
import VueRenderPlugin from "rete-vue-render-plugin";
import ContextMenuPlugin from "rete-context-menu-plugin";
import AreaPlugin from "rete-area-plugin";
import CommentPlugin from "rete-comment-plugin";
import HistoryPlugin from "rete-history-plugin";
import ConnectionMasteryPlugin from "rete-connection-mastery-plugin";
import components from "../editor/components";

export default Vue.extend({
  name: "GraphEditor",
  methods: {
    change(e) {
      console.log("change()")
      this.value = +e.target.value;
      this.update();
    },
    update() {
      console.log("update()")
      if (this.ikey)
          this.putData(this.ikey, this.value);
        this.emitter.trigger("process");
    },
    async initEditor(container) {
    // initEditor(container) {
      console.log("initEditor()");
      this.ID = "demo@0.1.0";

      this.editor = new Rete.NodeEditor(this.ID, container);
      this.editor.use(VueRenderPlugin);
      this.editor.use(ConnectionPlugin);
      this.editor.use(ContextMenuPlugin);
      this.editor.use(AreaPlugin);
      this.editor.use(CommentPlugin);
      this.editor.use(HistoryPlugin);
      this.editor.use(ConnectionMasteryPlugin);

      this.engine = new Rete.Engine(this.ID);

      components.list.map(c => {
        this.editor.register(c);
        this.engine.register(c);
      });

      // console.log("rete component 0 has name: " + components.list[0].name);
      let n1 = await components.list[0].createNode({ num: 2 });
      // console.log("created node: " + n1 + "from component: " + components.list[0].name);
      // console.log(JSON.stringify(n1))
      // console.log(Promise.resolve(n1));
    

      let n2 = await components.list[0].createNode({ num: 0 });
      let add = await components.list[1].createNode();

      n1.position = [80, 200];
      n2.position = [80, 400];
      add.position = [500, 240];

      console.log(this.editor);
      console.log(n1);

      this.editor.addNode(n1);
      this.editor.addNode(n2);
      this.editor.addNode(add);

      this.editor.connect(n1.outputs.get("num"), add.inputs.get("num"));
      this.editor.connect(n2.outputs.get("num"), add.inputs.get("num2"));

      this.editor.on(
        "process nodecreated noderemoved connectioncreated connectionremoved",
        async () => {
          console.log("editor.on()");
          await this.engine.abort();
          await this.engine.process(this.editor.toJSON());
        },
      );

      this.editor.view.resize();
      AreaPlugin.zoomAt(this.editor);
      this.editor.trigger("process");
    },
  },
  async mounted() {
    console.log("mounted()");
    // this.$nextTick(function() {
    //   console.log("mounted().$nextTick");
    //   // let container = this.$el;
    let container = this.$refs.rete;
    console.log(container)
    this.initEditor(container);
    // });
  },
});
</script>
<style>
#rete {
  width: 100%;
  height: 1000px;
}

.node .control input, .node .input-control input {
  width: 140px;
}

select, input {
  width: 100%;
  border-radius: 30px;
  background-color: white;
  padding: 2px 6px;
  border: 1px solid #999;
  font-size: 110%;
  width: 170px;
}
</style>