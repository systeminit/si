import Vue from "vue";
import Vuetify from "vuetify/lib";
import colors from "vuetify/lib/util/colors";

Vue.use(Vuetify);

export default new Vuetify({
  icons: {
    iconfont: "mdi",
  },
  theme: {
    dark: true,
    themes: {
      light: {
        primary: "#673ab7",
        secondary: "#9c27b0",
        accent: "#e91e63",
        error: "#ff5722",
        warning: "#ff9800",
        info: "#03a9f4",
        success: "#8bc34a",
      },
      dark: {
        primary: colors.grey.darken2,
        secondary: colors.orange.darken2,
        accent: colors.yellow.darken4,
        error: colors.red.darken2,
        warning: colors.orange.darken4,
        info: colors.cyan.darken4,
        success: colors.green.darken4,
        background: colors.shades.black,
        ctabackground: "#151515",
        ctabutton: "#676767",
        headertext: "#BBBBBB",
        anchor: colors.grey.lighten1,
      },
    },
  },
});
