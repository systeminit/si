"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
var _exportNames = {
  registry: true,
  ObjectTypes: true,
  Associations: true,
  BelongsTo: true,
  HasMany: true,
  variablesObjectForProperty: true,
  QueryArgs: true
};
Object.defineProperty(exports, "registry", {
  enumerable: true,
  get: function get() {
    return _registry.registry;
  }
});
Object.defineProperty(exports, "ObjectTypes", {
  enumerable: true,
  get: function get() {
    return _systemComponent.ObjectTypes;
  }
});
Object.defineProperty(exports, "Associations", {
  enumerable: true,
  get: function get() {
    return _associations.Associations;
  }
});
Object.defineProperty(exports, "BelongsTo", {
  enumerable: true,
  get: function get() {
    return _associations.BelongsTo;
  }
});
Object.defineProperty(exports, "HasMany", {
  enumerable: true,
  get: function get() {
    return _associations.HasMany;
  }
});
Object.defineProperty(exports, "variablesObjectForProperty", {
  enumerable: true,
  get: function get() {
    return _graphql.variablesObjectForProperty;
  }
});
Object.defineProperty(exports, "QueryArgs", {
  enumerable: true,
  get: function get() {
    return _graphql.QueryArgs;
  }
});

require("./loader");

var _registry = require("./registry");

var _prelude = require("./components/prelude");

Object.keys(_prelude).forEach(function (key) {
  if (key === "default" || key === "__esModule") return;
  if (Object.prototype.hasOwnProperty.call(_exportNames, key)) return;
  Object.defineProperty(exports, key, {
    enumerable: true,
    get: function get() {
      return _prelude[key];
    }
  });
});

var _systemComponent = require("./systemComponent");

var _associations = require("./systemObject/associations");

var _graphql = require("./systemObject/graphql");
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9pbmRleC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFPQTs7QUFDQTs7QUFDQTs7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFDQTs7QUFDQTs7QUFDQSIsInNvdXJjZXNDb250ZW50IjpbIi8vXG4vLyBUaGlzIHBhdGggaXMgcmVsYXRpdmUsIGJlY3Vhc2UgdGhpcyBwcm9qZWN0IGlzIHVzZWQgZGlyZWN0bHkgYnkgb3RoZXJcbi8vIHR5cGVzY3JpcHQgcHJvamVjdHMuIEl0IHN1Y2tzLCBJIGtub3csIGJ1dCBpdCBpcyB3aGF0IGl0IGlzLCBpZiB3ZSB3YW50XG4vLyB0byBhdm9pZCB1c2luZyBhIGJhYmVsL3dlYnBhY2sgc29sdXRpb24sIGFuZCByZWNvbXBpbGluZyB3aGVuZXZlciB0aGluZ3Ncbi8vIGNoYW5nZS5cbi8vXG5cbmltcG9ydCBcIi4vbG9hZGVyXCI7XG5leHBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuL3JlZ2lzdHJ5XCI7XG5leHBvcnQgKiBmcm9tIFwiLi9jb21wb25lbnRzL3ByZWx1ZGVcIjtcbmV4cG9ydCB7IE9iamVjdFR5cGVzIH0gZnJvbSBcIi4vc3lzdGVtQ29tcG9uZW50XCI7XG5leHBvcnQgeyBBc3NvY2lhdGlvbnMsIEJlbG9uZ3NUbywgSGFzTWFueSB9IGZyb20gXCIuL3N5c3RlbU9iamVjdC9hc3NvY2lhdGlvbnNcIjtcbmV4cG9ydCB7IHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5LCBRdWVyeUFyZ3MgfSBmcm9tIFwiLi9zeXN0ZW1PYmplY3QvZ3JhcGhxbFwiO1xuXG4iXX0=