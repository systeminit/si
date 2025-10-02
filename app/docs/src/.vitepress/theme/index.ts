import DefaultTheme from "vitepress/theme";
import Layout from "./Layout.vue";
import DocTabs from "./components/DocTabs.vue";
import TabPanel from "./components/TabPanel.vue";

export default {
  extends: DefaultTheme,
  Layout,
  enhanceApp({ app }) {
    app.component('DocTabs', DocTabs);
    app.component('TabPanel', TabPanel);
  }
}
