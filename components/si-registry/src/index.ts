//
// This path is relative, becuase this project is used directly by other
// typescript projects. It sucks, I know, but it is what it is, if we want
// to avoid using a babel/webpack solution, and recompiling whenever things
// change.
//

import "./loader";
export { registry } from "./registry";
export * from "./components/prelude";
export {
  ObjectTypes,
  BaseObject,
  SystemObject,
  ComponentObject,
  EntityObject,
  EntityEventObject,
} from "./systemComponent";
export { Associations, BelongsTo, HasMany } from "./systemObject/associations";
export { variablesObjectForProperty, QueryArgs } from "./systemObject/graphql";
