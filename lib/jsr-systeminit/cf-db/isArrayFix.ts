// Fix typescript issue where Array.isArray doesn't work for readonly arrays
// https://github.com/microsoft/TypeScript/issues/17002#issuecomment-2781717755
declare global {
  interface ArrayConstructor {
    // deno-lint-ignore no-explicit-any
    isArray(arg: ReadonlyArray<any> | any): arg is ReadonlyArray<any>;
  }
}
