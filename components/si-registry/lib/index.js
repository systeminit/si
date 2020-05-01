"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
var _exportNames = {
  registry: true,
  ObjectTypes: true,
  Associations: true,
  BelongsTo: true,
  HasMany: true
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9pbmRleC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBT0E7O0FBQ0E7O0FBQ0E7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBQ0E7O0FBQ0EiLCJzb3VyY2VzQ29udGVudCI6WyIvL1xuLy8gVGhpcyBwYXRoIGlzIHJlbGF0aXZlLCBiZWN1YXNlIHRoaXMgcHJvamVjdCBpcyB1c2VkIGRpcmVjdGx5IGJ5IG90aGVyXG4vLyB0eXBlc2NyaXB0IHByb2plY3RzLiBJdCBzdWNrcywgSSBrbm93LCBidXQgaXQgaXMgd2hhdCBpdCBpcywgaWYgd2Ugd2FudFxuLy8gdG8gYXZvaWQgdXNpbmcgYSBiYWJlbC93ZWJwYWNrIHNvbHV0aW9uLCBhbmQgcmVjb21waWxpbmcgd2hlbmV2ZXIgdGhpbmdzXG4vLyBjaGFuZ2UuXG4vL1xuXG5pbXBvcnQgXCIuL2xvYWRlclwiO1xuZXhwb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi9yZWdpc3RyeVwiO1xuZXhwb3J0ICogZnJvbSBcIi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5leHBvcnQgeyBPYmplY3RUeXBlcyB9IGZyb20gXCIuL3N5c3RlbUNvbXBvbmVudFwiO1xuZXhwb3J0IHsgQXNzb2NpYXRpb25zLCBCZWxvbmdzVG8sIEhhc01hbnkgfSBmcm9tIFwiLi9zeXN0ZW1PYmplY3QvYXNzb2NpYXRpb25zXCI7XG4iXX0=