<template>
  <div id="rete" ref="rete"></div>
</template>

<script lang="ts">
import Vue from "vue";

import Rete from "rete";
import ConnectionPlugin from "rete-connection-plugin";
import VueRenderPlugin from "rete-vue-render-plugin";
import ContextMenuPlugin from "rete-context-menu-plugin";
import AreaPlugin from "rete-area-plugin";
import CommentPlugin from "rete-comment-plugin";
import HistoryPlugin from "rete-history-plugin";
import ConnectionMasteryPlugin from "rete-connection-mastery-plugin";
import { Output, Input } from "rete";

import components from "../graph-editor/components";

export default Vue.extend({
  name: "GraphEditor",
  props: ["readonly", "emitter", "ikey", "getData", "putData"],
  data: function() {
    return {
      value: 0,
    };
  },
  methods: {
    change(e) {
      this.value = +e.target.value;
      this.update();
    },
    update() {
      if (this.ikey) this.putData(this.ikey, this.value);
      this.emitter.trigger("process");
    },
    eventHandler(e, engine, editor) {
      engine.abort();
      engine.process(editor.toJSON());
    },
    async initEditor(container) {
      let ID = "demo@0.1.0";

      let editor = new Rete.NodeEditor(ID, container);
      editor.use(VueRenderPlugin);
      editor.use(ConnectionPlugin);
      editor.use(ContextMenuPlugin);
      editor.use(AreaPlugin);
      editor.use(CommentPlugin);
      editor.use(HistoryPlugin);
      editor.use(ConnectionMasteryPlugin);

      let engine = new Rete.Engine(ID);

      components.list.map(c => {
        editor.register(c);
        engine.register(c);
      });

      // Create default nodes
      let n1 = await components.list[0].createNode({ num: 2 });
      let n2 = await components.list[0].createNode({ num: 0 });
      let add = await components.list[1].createNode();

      n1.position = [80, 200];
      n2.position = [80, 400];
      add.position = [500, 240];

      editor.addNode(n1);
      editor.addNode(n2);
      editor.addNode(add);

      editor.connect(
        n1.outputs.get("num") as Output,
        add.inputs.get("num") as Input,
      );
      editor.connect(
        n2.outputs.get("num") as Output,
        add.inputs.get("num2") as Input,
      );
      // await this.createExampleNode(components, editor);

      editor.on("process", async () => {
        await this.eventHandler("process", engine, editor);
      });

      editor.on("nodecreated", async () => {
        await this.eventHandler("nodecreated", engine, editor);
      });

      editor.on("noderemoved", async () => {
        await this.eventHandler("noderemoved", engine, editor);
      });

      editor.on("connectioncreated", async () => {
        await this.eventHandler("connectioncreated", engine, editor);
      });

      editor.on("connectionremoved", async () => {
        await this.eventHandler("connectionremoved", engine, editor);
      });

      editor.view.resize();
      AreaPlugin.zoomAt(editor);
      editor.trigger("process");
    },
    //   createExampleNode(components: any, editor: any) {
    //     // Create default nodes
    //     let n1 = components.list[0].createNode({ num: 2 });
    //     let n2 = components.list[0].createNode({ num: 0 });
    //     let add = components.list[1].createNode();

    //     n1.position = [80, 200];
    //     n2.position = [80, 400];
    //     add.position = [500, 240];

    //     editor.addNode(n1);
    //     editor.addNode(n2);
    //     editor.addNode(add);

    //     editor.connect(
    //       n1.outputs.get("num") as Output,
    //       add.inputs.get("num") as Input,
    //     );
    //     editor.connect(
    //       n2.outputs.get("num") as Output,
    //       add.inputs.get("num2") as Input,
    //     );
    //   },
  },
  async mounted() {
    let container = this.$refs.rete;
    this.initEditor(container);
  },
});
</script>
<style>
#rete {
  width: 100%;
  height: 512px;
}

.node.control input,
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
