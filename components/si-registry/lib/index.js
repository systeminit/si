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
  variablesObjectForProperty: true
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9pbmRleC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQU9BOztBQUNBOztBQUNBOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBOztBQUNBOztBQUNBOztBQUNBIiwic291cmNlc0NvbnRlbnQiOlsiLy9cbi8vIFRoaXMgcGF0aCBpcyByZWxhdGl2ZSwgYmVjdWFzZSB0aGlzIHByb2plY3QgaXMgdXNlZCBkaXJlY3RseSBieSBvdGhlclxuLy8gdHlwZXNjcmlwdCBwcm9qZWN0cy4gSXQgc3Vja3MsIEkga25vdywgYnV0IGl0IGlzIHdoYXQgaXQgaXMsIGlmIHdlIHdhbnRcbi8vIHRvIGF2b2lkIHVzaW5nIGEgYmFiZWwvd2VicGFjayBzb2x1dGlvbiwgYW5kIHJlY29tcGlsaW5nIHdoZW5ldmVyIHRoaW5nc1xuLy8gY2hhbmdlLlxuLy9cblxuaW1wb3J0IFwiLi9sb2FkZXJcIjtcbmV4cG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4vcmVnaXN0cnlcIjtcbmV4cG9ydCAqIGZyb20gXCIuL2NvbXBvbmVudHMvcHJlbHVkZVwiO1xuZXhwb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi9zeXN0ZW1Db21wb25lbnRcIjtcbmV4cG9ydCB7IEFzc29jaWF0aW9ucywgQmVsb25nc1RvLCBIYXNNYW55IH0gZnJvbSBcIi4vc3lzdGVtT2JqZWN0L2Fzc29jaWF0aW9uc1wiO1xuZXhwb3J0IHsgdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkgfSBmcm9tIFwiLi9zeXN0ZW1PYmplY3QvZ3JhcGhxbFwiOyJdfQ==