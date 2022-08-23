import tailwind from "tailwindcss";
import autoprefixer from "autoprefixer";
import tailwindConfig from "./tailwind.config.mjs";

export default {
  plugins: [tailwind(tailwindConfig), autoprefixer],
};
