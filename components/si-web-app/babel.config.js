/* eslint-disable no-var */
module.exports = api => {
  var isProd = api.cache.invalidate(
    () => process.env.NODE_ENV === "production",
  );
  var plugins = [];
  if (isProd) {
    plugins.push(["transform-remove-console"]);
  }
  return {
    presets: ["@vue/cli-plugin-babel/preset"],
    plugins,
  };
};

//module.exports = {
//  presets: ["@vue/cli-plugin-babel/preset"],
//};
