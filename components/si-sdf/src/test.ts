import _ from "https://deno.land/x/lodash@4.17.15-es/lodash.js";

  const monkey: number[] = [1, 2, 3, 4];
  _.map(monkey, (foo: number) => {
    return foo * 2;
  });
  // @ts-ignore
  Deno.core.print(`${monkey}\n`);
