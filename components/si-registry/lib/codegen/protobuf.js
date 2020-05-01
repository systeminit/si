"use strict";

function _typeof(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.ProtobufFormatter = void 0;

var _ejs = _interopRequireDefault(require("ejs"));

var _attrList = require("../attrList");

var _enum = require("../prop/enum");

var PropPrelude = _interopRequireWildcard(require("../components/prelude"));

var _changeCase = require("change-case");

function _getRequireWildcardCache() { if (typeof WeakMap !== "function") return null; var cache = new WeakMap(); _getRequireWildcardCache = function _getRequireWildcardCache() { return cache; }; return cache; }

function _interopRequireWildcard(obj) { if (obj && obj.__esModule) { return obj; } if (obj === null || _typeof(obj) !== "object" && typeof obj !== "function") { return { "default": obj }; } var cache = _getRequireWildcardCache(); if (cache && cache.has(obj)) { return cache.get(obj); } var newObj = {}; var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor; for (var key in obj) { if (Object.prototype.hasOwnProperty.call(obj, key)) { var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null; if (desc && (desc.get || desc.set)) { Object.defineProperty(newObj, key, desc); } else { newObj[key] = obj[key]; } } } newObj["default"] = obj; if (cache) { cache.set(obj, newObj); } return newObj; }

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { "default": obj }; }

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

var ProtobufFormatter = /*#__PURE__*/function () {
  function ProtobufFormatter(systemObjects) {
    _classCallCheck(this, ProtobufFormatter);

    _defineProperty(this, "systemObjects", void 0);

    _defineProperty(this, "recurseKinds", ["object"]);

    if (systemObjects.length == 0) {
      throw "You must provide objects to generate";
    }

    this.systemObjects = systemObjects;
  }

  _createClass(ProtobufFormatter, [{
    key: "first",
    value: function first() {
      return this.systemObjects[0];
    }
  }, {
    key: "protobufPackageName",
    value: function protobufPackageName() {
      return "si.".concat((0, _changeCase.snakeCase)(this.first().serviceName));
    }
  }, {
    key: "protobufServices",
    value: function protobufServices() {
      var results = [];

      if (this.systemObjects.filter(function (obj) {
        return obj.methodsProp.properties.length > 0;
      }).length > 0) {
        results.push("service ".concat((0, _changeCase.pascalCase)(this.first().serviceName), " {"));

        var _iterator = _createForOfIteratorHelper(this.systemObjects),
            _step;

        try {
          for (_iterator.s(); !(_step = _iterator.n()).done;) {
            var object = _step.value;

            var _iterator2 = _createForOfIteratorHelper(object.methods.attrs),
                _step2;

            try {
              for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
                var method = _step2.value;
                var methodName = (0, _changeCase.pascalCase)(method.parentName) + (0, _changeCase.pascalCase)(method.name);
                results.push("  rpc ".concat(methodName, "(").concat(methodName, "Request) returns (").concat(methodName, "Reply);"));
              }
            } catch (err) {
              _iterator2.e(err);
            } finally {
              _iterator2.f();
            }
          }
        } catch (err) {
          _iterator.e(err);
        } finally {
          _iterator.f();
        }

        results.push("}");
        return results.join("\n");
      }

      return "// No Services";
    }
  }, {
    key: "protobufMessages",
    value: function protobufMessages() {
      var results = [];

      var _iterator3 = _createForOfIteratorHelper(this.systemObjects),
          _step3;

      try {
        for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
          var object = _step3.value;
          results.push(this.protobufMessageForPropObject(object.rootProp));

          if (object.methodsProp.properties.length) {
            var _iterator4 = _createForOfIteratorHelper(object.methodsProp.properties.attrs),
                _step4;

            try {
              for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
                var methodHolder = _step4.value;

                if (methodHolder instanceof PropPrelude.PropMethod || methodHolder instanceof PropPrelude.PropAction) {
                  results.push(this.protobufMessageForPropObject(methodHolder.request));
                  results.push(this.protobufMessageForPropObject(methodHolder.reply));
                } else {
                  throw "Error generating protobuf - non method/action prop found on ".concat(object.typeName);
                }
              }
            } catch (err) {
              _iterator4.e(err);
            } finally {
              _iterator4.f();
            }
          }
        }
      } catch (err) {
        _iterator3.e(err);
      } finally {
        _iterator3.f();
      }

      return results.join("\n");
    }
  }, {
    key: "protobufImportForProp",
    value: function protobufImportForProp(prop) {
      if (prop instanceof PropPrelude.PropLink) {
        var propOwner = prop.lookupObject();
        var pathName = "si-registry/proto/si.";

        if (propOwner.serviceName) {
          pathName = pathName + (0, _changeCase.snakeCase)(propOwner.serviceName) + ".proto";
        } else {
          pathName = pathName + (0, _changeCase.snakeCase)(propOwner.typeName) + ".proto";
        }

        return pathName;
      } else {
        return "";
      }
    }
  }, {
    key: "protobufTypeForProp",
    value: function protobufTypeForProp(prop) {
      if (prop instanceof PropPrelude.PropBool) {
        return "google.protobuf.BoolValue";
      } else if (prop instanceof PropPrelude.PropCode) {
        return "google.protobuf.StringValue";
      } else if (prop instanceof PropPrelude.PropEnum) {
        return "".concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name));
      } else if (prop instanceof PropPrelude.PropLink) {
        var realProp = prop.lookupMyself();

        if (realProp instanceof PropPrelude.PropObject || realProp instanceof PropPrelude.PropEnum) {
          var propOwner = prop.lookupObject();
          var pathName = "si.";

          if (propOwner.serviceName) {
            pathName = pathName + (0, _changeCase.snakeCase)(propOwner.serviceName);
          } else {
            pathName = pathName + (0, _changeCase.snakeCase)(propOwner.typeName);
          }

          return "".concat(pathName, ".").concat((0, _changeCase.pascalCase)(realProp.parentName)).concat((0, _changeCase.pascalCase)(realProp.name));
        } else {
          return this.protobufTypeForProp(realProp);
        }
      } else if (prop instanceof PropPrelude.PropMap) {
        return "map<string, string>";
      } else if (prop instanceof PropPrelude.PropNumber) {
        if (prop.numberKind == "int32") {
          return "google.protobuf.Int32Value";
        } else if (prop.numberKind == "uint32") {
          return "google.protobuf.UInt32Value";
        } else if (prop.numberKind == "int64") {
          return "google.protobuf.Int64Value";
        } else if (prop.numberKind == "uint64") {
          return "google.protobuf.UInt64Value";
        }
      } else if (prop instanceof PropPrelude.PropObject) {
        return "".concat(this.protobufPackageName(), ".").concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name));
      } else if (prop instanceof PropPrelude.PropMethod) {
        return "".concat(this.protobufPackageName(), ".").concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name));
      } else if (prop instanceof PropPrelude.PropAction) {
        return "".concat(this.protobufPackageName(), ".").concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name));
      } else if (prop instanceof PropPrelude.PropSelect || prop instanceof PropPrelude.PropText) {
        return "google.protobuf.StringValue";
      } else {
        // @ts-ignore
        throw "Unknown property type for rendering protobuf! Fix me: ".concat(prop.kind());
      }
    }
  }, {
    key: "protobufDefinitionForProp",
    value: function protobufDefinitionForProp(prop, inputNumber) {
      var repeated;

      if (prop.repeated) {
        repeated = "repeated ";
      } else {
        repeated = "";
      }

      return "".concat(repeated).concat(this.protobufTypeForProp(prop), " ").concat((0, _changeCase.snakeCase)(prop.name), " = ").concat(inputNumber, ";");
    }
  }, {
    key: "protobufMessageForPropObject",
    value: function protobufMessageForPropObject(prop) {
      var results = [];

      if (prop instanceof _enum.PropEnum) {
        var enumCount = 0;
        results.push("enum ".concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name), " {"));
        results.push("  ".concat((0, _changeCase.constantCase)(this.protobufTypeForProp(prop)), "_UNKNOWN = ").concat(enumCount, ";"));

        var _iterator5 = _createForOfIteratorHelper(prop.variants),
            _step5;

        try {
          for (_iterator5.s(); !(_step5 = _iterator5.n()).done;) {
            var variant = _step5.value;
            enumCount = enumCount + 1;
            results.push("  ".concat((0, _changeCase.constantCase)(this.protobufTypeForProp(prop)), "_").concat((0, _changeCase.constantCase)(variant), " = ").concat(enumCount, ";"));
          }
        } catch (err) {
          _iterator5.e(err);
        } finally {
          _iterator5.f();
        }

        results.push("}");
        return results.join("\n");
      }

      var _iterator6 = _createForOfIteratorHelper(prop.bagNames()),
          _step6;

      try {
        for (_iterator6.s(); !(_step6 = _iterator6.n()).done;) {
          var bag = _step6.value;

          var _iterator7 = _createForOfIteratorHelper(prop[bag].attrs),
              _step7;

          try {
            for (_iterator7.s(); !(_step7 = _iterator7.n()).done;) {
              var childProp = _step7.value;

              if (childProp instanceof _attrList.PropObject || childProp instanceof _enum.PropEnum) {
                results.push(this.protobufMessageForPropObject(childProp));
              }
            }
          } catch (err) {
            _iterator7.e(err);
          } finally {
            _iterator7.f();
          }

          var messageName = "".concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name));
          results.push("message ".concat(messageName, " {"));
          var universalBase = 0;
          var customBase = 1000;

          for (var index in prop[bag].attrs) {
            var p = prop[bag].attrs[index];

            if (p.universal) {
              universalBase = universalBase + 1;
              results.push("  " + this.protobufDefinitionForProp(p, universalBase));
            } else {
              customBase = customBase + 1;
              results.push("  " + this.protobufDefinitionForProp(p, customBase));
            }
          }

          results.push("}");
        }
      } catch (err) {
        _iterator6.e(err);
      } finally {
        _iterator6.f();
      }

      return results.join("\n");
    }
  }, {
    key: "protobufImports",
    value: function protobufImports() {
      var results = []; // This creates a newline!

      var resultSet = this.protobufImportWalk(this.systemObjects.map(function (v) {
        return v.rootProp;
      }));

      var _iterator8 = _createForOfIteratorHelper(resultSet.values()),
          _step8;

      try {
        for (_iterator8.s(); !(_step8 = _iterator8.n()).done;) {
          var line = _step8.value;
          results.push("import \"".concat(line, "\";"));
        }
      } catch (err) {
        _iterator8.e(err);
      } finally {
        _iterator8.f();
      }

      if (results.length > 0) {
        return results.join("\n");
      } else {
        return "// No Imports";
      }
    }
  }, {
    key: "protobufImportWalk",
    value: function protobufImportWalk(props) {
      var result = new Set();

      var _iterator9 = _createForOfIteratorHelper(props),
          _step9;

      try {
        for (_iterator9.s(); !(_step9 = _iterator9.n()).done;) {
          var prop = _step9.value;

          if (prop.kind() == "link") {
            var importPath = this.protobufImportForProp(prop);

            if (importPath && !importPath.startsWith("si-registry/proto/".concat(this.protobufPackageName()))) {
              result.add(importPath);
            }
          } else {
            result.add("google/protobuf/wrappers.proto");
          }

          if (this.recurseKinds.includes(prop.kind())) {
            var _iterator10 = _createForOfIteratorHelper(prop.bagNames()),
                _step10;

            try {
              for (_iterator10.s(); !(_step10 = _iterator10.n()).done;) {
                var bag = _step10.value;

                var _iterator11 = _createForOfIteratorHelper(prop[bag].attrs),
                    _step11;

                try {
                  for (_iterator11.s(); !(_step11 = _iterator11.n()).done;) {
                    var internalProp = _step11.value;
                    var newSet = this.protobufImportWalk([internalProp]);

                    var _iterator12 = _createForOfIteratorHelper(newSet.values()),
                        _step12;

                    try {
                      for (_iterator12.s(); !(_step12 = _iterator12.n()).done;) {
                        var item = _step12.value;
                        result.add(item);
                      }
                    } catch (err) {
                      _iterator12.e(err);
                    } finally {
                      _iterator12.f();
                    }
                  }
                } catch (err) {
                  _iterator11.e(err);
                } finally {
                  _iterator11.f();
                }
              }
            } catch (err) {
              _iterator10.e(err);
            } finally {
              _iterator10.f();
            }
          }
        }
      } catch (err) {
        _iterator9.e(err);
      } finally {
        _iterator9.f();
      }

      return result;
    }
  }, {
    key: "generateString",
    value: function generateString() {
      return _ejs["default"].render("<%- include('protobuf/proto', { fmt }) %>", {
        fmt: this
      }, {
        filename: __filename
      });
    }
  }]);

  return ProtobufFormatter;
}(); //export class CodegenProtobuf {
//  component: Component;
//
//  constructor(component: Component) {
//    this.component = component;
//  }
//
//  generateString(): string {
//    return ejs.render(
//      "<%- include('protobuf/full', { component: component }) %>",
//      {
//        component: this.component,
//      },
//      {
//        filename: __filename,
//      },
//    );
//  }
//}


exports.ProtobufFormatter = ProtobufFormatter;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3Byb3RvYnVmLnRzIl0sIm5hbWVzIjpbIlByb3RvYnVmRm9ybWF0dGVyIiwic3lzdGVtT2JqZWN0cyIsImxlbmd0aCIsImZpcnN0Iiwic2VydmljZU5hbWUiLCJyZXN1bHRzIiwiZmlsdGVyIiwib2JqIiwibWV0aG9kc1Byb3AiLCJwcm9wZXJ0aWVzIiwicHVzaCIsIm9iamVjdCIsIm1ldGhvZHMiLCJhdHRycyIsIm1ldGhvZCIsIm1ldGhvZE5hbWUiLCJwYXJlbnROYW1lIiwibmFtZSIsImpvaW4iLCJwcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0Iiwicm9vdFByb3AiLCJtZXRob2RIb2xkZXIiLCJQcm9wUHJlbHVkZSIsIlByb3BNZXRob2QiLCJQcm9wQWN0aW9uIiwicmVxdWVzdCIsInJlcGx5IiwidHlwZU5hbWUiLCJwcm9wIiwiUHJvcExpbmsiLCJwcm9wT3duZXIiLCJsb29rdXBPYmplY3QiLCJwYXRoTmFtZSIsIlByb3BCb29sIiwiUHJvcENvZGUiLCJQcm9wRW51bSIsInJlYWxQcm9wIiwibG9va3VwTXlzZWxmIiwiUHJvcE9iamVjdCIsInByb3RvYnVmVHlwZUZvclByb3AiLCJQcm9wTWFwIiwiUHJvcE51bWJlciIsIm51bWJlcktpbmQiLCJwcm90b2J1ZlBhY2thZ2VOYW1lIiwiUHJvcFNlbGVjdCIsIlByb3BUZXh0Iiwia2luZCIsImlucHV0TnVtYmVyIiwicmVwZWF0ZWQiLCJlbnVtQ291bnQiLCJ2YXJpYW50cyIsInZhcmlhbnQiLCJiYWdOYW1lcyIsImJhZyIsImNoaWxkUHJvcCIsIm1lc3NhZ2VOYW1lIiwidW5pdmVyc2FsQmFzZSIsImN1c3RvbUJhc2UiLCJpbmRleCIsInAiLCJ1bml2ZXJzYWwiLCJwcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wIiwicmVzdWx0U2V0IiwicHJvdG9idWZJbXBvcnRXYWxrIiwibWFwIiwidiIsInZhbHVlcyIsImxpbmUiLCJwcm9wcyIsInJlc3VsdCIsIlNldCIsImltcG9ydFBhdGgiLCJwcm90b2J1ZkltcG9ydEZvclByb3AiLCJzdGFydHNXaXRoIiwiYWRkIiwicmVjdXJzZUtpbmRzIiwiaW5jbHVkZXMiLCJpbnRlcm5hbFByb3AiLCJuZXdTZXQiLCJpdGVtIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiLCJfX2ZpbGVuYW1lIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFFQTs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztJQUVhQSxpQjtBQUtYLDZCQUFZQyxhQUFaLEVBQTBDO0FBQUE7O0FBQUE7O0FBQUEsMENBRjNCLENBQUMsUUFBRCxDQUUyQjs7QUFDeEMsUUFBSUEsYUFBYSxDQUFDQyxNQUFkLElBQXdCLENBQTVCLEVBQStCO0FBQzdCLFlBQU0sc0NBQU47QUFDRDs7QUFDRCxTQUFLRCxhQUFMLEdBQXFCQSxhQUFyQjtBQUNEOzs7OzRCQUVvQjtBQUNuQixhQUFPLEtBQUtBLGFBQUwsQ0FBbUIsQ0FBbkIsQ0FBUDtBQUNEOzs7MENBRTZCO0FBQzVCLDBCQUFhLDJCQUFVLEtBQUtFLEtBQUwsR0FBYUMsV0FBdkIsQ0FBYjtBQUNEOzs7dUNBRTBCO0FBQ3pCLFVBQU1DLE9BQU8sR0FBRyxFQUFoQjs7QUFDQSxVQUNFLEtBQUtKLGFBQUwsQ0FBbUJLLE1BQW5CLENBQTBCLFVBQUFDLEdBQUc7QUFBQSxlQUFJQSxHQUFHLENBQUNDLFdBQUosQ0FBZ0JDLFVBQWhCLENBQTJCUCxNQUEzQixHQUFvQyxDQUF4QztBQUFBLE9BQTdCLEVBQ0dBLE1BREgsR0FDWSxDQUZkLEVBR0U7QUFDQUcsUUFBQUEsT0FBTyxDQUFDSyxJQUFSLG1CQUF3Qiw0QkFBVyxLQUFLUCxLQUFMLEdBQWFDLFdBQXhCLENBQXhCOztBQURBLG1EQUVxQixLQUFLSCxhQUYxQjtBQUFBOztBQUFBO0FBRUEsOERBQXlDO0FBQUEsZ0JBQTlCVSxNQUE4Qjs7QUFBQSx3REFDbEJBLE1BQU0sQ0FBQ0MsT0FBUCxDQUFlQyxLQURHO0FBQUE7O0FBQUE7QUFD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L3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IGVqcyBmcm9tIFwiZWpzXCI7XG5pbXBvcnQgeyBQcm9wcywgUHJvcE9iamVjdCB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuaW1wb3J0IHsgUHJvcEVudW0gfSBmcm9tIFwiLi4vcHJvcC9lbnVtXCI7XG5pbXBvcnQgKiBhcyBQcm9wUHJlbHVkZSBmcm9tIFwiLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5cbmltcG9ydCB7IHNuYWtlQ2FzZSwgcGFzY2FsQ2FzZSwgY29uc3RhbnRDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5cbmV4cG9ydCBjbGFzcyBQcm90b2J1ZkZvcm1hdHRlciB7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgcmVjdXJzZUtpbmRzID0gW1wib2JqZWN0XCJdO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW10pIHtcbiAgICBpZiAoc3lzdGVtT2JqZWN0cy5sZW5ndGggPT0gMCkge1xuICAgICAgdGhyb3cgXCJZb3UgbXVzdCBwcm92aWRlIG9iamVjdHMgdG8gZ2VuZXJhdGVcIjtcbiAgICB9XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gc3lzdGVtT2JqZWN0cztcbiAgfVxuXG4gIGZpcnN0KCk6IE9iamVjdFR5cGVzIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzWzBdO1xuICB9XG5cbiAgcHJvdG9idWZQYWNrYWdlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgc2kuJHtzbmFrZUNhc2UodGhpcy5maXJzdCgpLnNlcnZpY2VOYW1lKX1gO1xuICB9XG5cbiAgcHJvdG9idWZTZXJ2aWNlcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdHMuZmlsdGVyKG9iaiA9PiBvYmoubWV0aG9kc1Byb3AucHJvcGVydGllcy5sZW5ndGggPiAwKVxuICAgICAgICAubGVuZ3RoID4gMFxuICAgICkge1xuICAgICAgcmVzdWx0cy5wdXNoKGBzZXJ2aWNlICR7cGFzY2FsQ2FzZSh0aGlzLmZpcnN0KCkuc2VydmljZU5hbWUpfSB7YCk7XG4gICAgICBmb3IgKGNvbnN0IG9iamVjdCBvZiB0aGlzLnN5c3RlbU9iamVjdHMpIHtcbiAgICAgICAgZm9yIChjb25zdCBtZXRob2Qgb2Ygb2JqZWN0Lm1ldGhvZHMuYXR0cnMpIHtcbiAgICAgICAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgICAgICAgIHBhc2NhbENhc2UobWV0aG9kLnBhcmVudE5hbWUpICsgcGFzY2FsQ2FzZShtZXRob2QubmFtZSk7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICAgICAgYCAgcnBjICR7bWV0aG9kTmFtZX0oJHttZXRob2ROYW1lfVJlcXVlc3QpIHJldHVybnMgKCR7bWV0aG9kTmFtZX1SZXBseSk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgICByZXN1bHRzLnB1c2goYH1gKTtcbiAgICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gICAgfVxuICAgIHJldHVybiBcIi8vIE5vIFNlcnZpY2VzXCI7XG4gIH1cblxuICBwcm90b2J1Zk1lc3NhZ2VzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGZvciAoY29uc3Qgb2JqZWN0IG9mIHRoaXMuc3lzdGVtT2JqZWN0cykge1xuICAgICAgcmVzdWx0cy5wdXNoKHRoaXMucHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChvYmplY3Qucm9vdFByb3ApKTtcbiAgICAgIGlmIChvYmplY3QubWV0aG9kc1Byb3AucHJvcGVydGllcy5sZW5ndGgpIHtcbiAgICAgICAgZm9yIChjb25zdCBtZXRob2RIb2xkZXIgb2Ygb2JqZWN0Lm1ldGhvZHNQcm9wLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgICBpZiAoXG4gICAgICAgICAgICBtZXRob2RIb2xkZXIgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHx8XG4gICAgICAgICAgICBtZXRob2RIb2xkZXIgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uXG4gICAgICAgICAgKSB7XG4gICAgICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgICAgIHRoaXMucHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChtZXRob2RIb2xkZXIucmVxdWVzdCksXG4gICAgICAgICAgICApO1xuICAgICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMucHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChtZXRob2RIb2xkZXIucmVwbHkpKTtcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgdGhyb3cgYEVycm9yIGdlbmVyYXRpbmcgcHJvdG9idWYgLSBub24gbWV0aG9kL2FjdGlvbiBwcm9wIGZvdW5kIG9uICR7b2JqZWN0LnR5cGVOYW1lfWA7XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBwcm90b2J1ZkltcG9ydEZvclByb3AocHJvcDogUHJvcHMpOiBzdHJpbmcge1xuICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICBsZXQgcGF0aE5hbWUgPSBcInNpLXJlZ2lzdHJ5L3Byb3RvL3NpLlwiO1xuICAgICAgaWYgKHByb3BPd25lci5zZXJ2aWNlTmFtZSkge1xuICAgICAgICBwYXRoTmFtZSA9IHBhdGhOYW1lICsgc25ha2VDYXNlKHByb3BPd25lci5zZXJ2aWNlTmFtZSkgKyBcIi5wcm90b1wiO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcGF0aE5hbWUgPSBwYXRoTmFtZSArIHNuYWtlQ2FzZShwcm9wT3duZXIudHlwZU5hbWUpICsgXCIucHJvdG9cIjtcbiAgICAgIH1cbiAgICAgIHJldHVybiBwYXRoTmFtZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9XG5cbiAgcHJvdG9idWZUeXBlRm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQm9vbCkge1xuICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLkJvb2xWYWx1ZVwiO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlKSB7XG4gICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuU3RyaW5nVmFsdWVcIjtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wRW51bSkge1xuICAgICAgcmV0dXJuIGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICBpZiAoXG4gICAgICAgIHJlYWxQcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCB8fFxuICAgICAgICByZWFsUHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BFbnVtXG4gICAgICApIHtcbiAgICAgICAgY29uc3QgcHJvcE93bmVyID0gcHJvcC5sb29rdXBPYmplY3QoKTtcbiAgICAgICAgbGV0IHBhdGhOYW1lID0gXCJzaS5cIjtcbiAgICAgICAgaWYgKHByb3BPd25lci5zZXJ2aWNlTmFtZSkge1xuICAgICAgICAgIHBhdGhOYW1lID0gcGF0aE5hbWUgKyBzbmFrZUNhc2UocHJvcE93bmVyLnNlcnZpY2VOYW1lKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IHBhdGhOYW1lICsgc25ha2VDYXNlKHByb3BPd25lci50eXBlTmFtZSk7XG4gICAgICAgIH1cbiAgICAgICAgcmV0dXJuIGAke3BhdGhOYW1lfS4ke3Bhc2NhbENhc2UocmVhbFByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICAgIHJlYWxQcm9wLm5hbWUsXG4gICAgICAgICl9YDtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiB0aGlzLnByb3RvYnVmVHlwZUZvclByb3AocmVhbFByb3ApO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNYXApIHtcbiAgICAgIHJldHVybiBcIm1hcDxzdHJpbmcsIHN0cmluZz5cIjtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTnVtYmVyKSB7XG4gICAgICBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50MzJcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuSW50MzJWYWx1ZVwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1aW50MzJcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuVUludDMyVmFsdWVcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50NjRcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuSW50NjRWYWx1ZVwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1aW50NjRcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuVUludDY0VmFsdWVcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICByZXR1cm4gYCR7dGhpcy5wcm90b2J1ZlBhY2thZ2VOYW1lKCl9LiR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5wYXJlbnROYW1lLFxuICAgICAgKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIHJldHVybiBgJHt0aGlzLnByb3RvYnVmUGFja2FnZU5hbWUoKX0uJHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLnBhcmVudE5hbWUsXG4gICAgICApfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbikge1xuICAgICAgcmV0dXJuIGAke3RoaXMucHJvdG9idWZQYWNrYWdlTmFtZSgpfS4ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AucGFyZW50TmFtZSxcbiAgICAgICl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BTZWxlY3QgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wVGV4dFxuICAgICkge1xuICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLlN0cmluZ1ZhbHVlXCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIHRocm93IGBVbmtub3duIHByb3BlcnR5IHR5cGUgZm9yIHJlbmRlcmluZyBwcm90b2J1ZiEgRml4IG1lOiAke3Byb3Aua2luZCgpfWA7XG4gICAgfVxuICB9XG5cbiAgcHJvdG9idWZEZWZpbml0aW9uRm9yUHJvcChwcm9wOiBQcm9wcywgaW5wdXROdW1iZXI6IG51bWJlcik6IHN0cmluZyB7XG4gICAgbGV0IHJlcGVhdGVkOiBzdHJpbmc7XG4gICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgIHJlcGVhdGVkID0gXCJyZXBlYXRlZCBcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgcmVwZWF0ZWQgPSBcIlwiO1xuICAgIH1cbiAgICByZXR1cm4gYCR7cmVwZWF0ZWR9JHt0aGlzLnByb3RvYnVmVHlwZUZvclByb3AocHJvcCl9ICR7c25ha2VDYXNlKFxuICAgICAgcHJvcC5uYW1lLFxuICAgICl9ID0gJHtpbnB1dE51bWJlcn07YDtcbiAgfVxuXG4gIHByb3RvYnVmTWVzc2FnZUZvclByb3BPYmplY3QocHJvcDogUHJvcE9iamVjdCB8IFByb3BFbnVtKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107XG5cbiAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BFbnVtKSB7XG4gICAgICBsZXQgZW51bUNvdW50ID0gMDtcbiAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgYGVudW0gJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9IHtgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgYCAgJHtjb25zdGFudENhc2UoXG4gICAgICAgICAgdGhpcy5wcm90b2J1ZlR5cGVGb3JQcm9wKHByb3ApLFxuICAgICAgICApfV9VTktOT1dOID0gJHtlbnVtQ291bnR9O2AsXG4gICAgICApO1xuICAgICAgZm9yIChjb25zdCB2YXJpYW50IG9mIHByb3AudmFyaWFudHMpIHtcbiAgICAgICAgZW51bUNvdW50ID0gZW51bUNvdW50ICsgMTtcbiAgICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICAgIGAgICR7Y29uc3RhbnRDYXNlKHRoaXMucHJvdG9idWZUeXBlRm9yUHJvcChwcm9wKSl9XyR7Y29uc3RhbnRDYXNlKFxuICAgICAgICAgICAgdmFyaWFudCxcbiAgICAgICAgICApfSA9ICR7ZW51bUNvdW50fTtgLFxuICAgICAgICApO1xuICAgICAgfVxuICAgICAgcmVzdWx0cy5wdXNoKFwifVwiKTtcbiAgICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gICAgfVxuXG4gICAgZm9yIChjb25zdCBiYWcgb2YgcHJvcC5iYWdOYW1lcygpKSB7XG4gICAgICBmb3IgKGNvbnN0IGNoaWxkUHJvcCBvZiBwcm9wW2JhZ10uYXR0cnMpIHtcbiAgICAgICAgaWYgKGNoaWxkUHJvcCBpbnN0YW5jZW9mIFByb3BPYmplY3QgfHwgY2hpbGRQcm9wIGluc3RhbmNlb2YgUHJvcEVudW0pIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2godGhpcy5wcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KGNoaWxkUHJvcCkpO1xuICAgICAgICB9XG4gICAgICB9XG5cbiAgICAgIGNvbnN0IG1lc3NhZ2VOYW1lID0gYCR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX1gO1xuICAgICAgcmVzdWx0cy5wdXNoKGBtZXNzYWdlICR7bWVzc2FnZU5hbWV9IHtgKTtcblxuICAgICAgbGV0IHVuaXZlcnNhbEJhc2UgPSAwO1xuICAgICAgbGV0IGN1c3RvbUJhc2UgPSAxMDAwO1xuICAgICAgZm9yIChjb25zdCBpbmRleCBpbiBwcm9wW2JhZ10uYXR0cnMpIHtcbiAgICAgICAgY29uc3QgcCA9IHByb3BbYmFnXS5hdHRyc1tpbmRleF07XG5cbiAgICAgICAgaWYgKHAudW5pdmVyc2FsKSB7XG4gICAgICAgICAgdW5pdmVyc2FsQmFzZSA9IHVuaXZlcnNhbEJhc2UgKyAxO1xuICAgICAgICAgIHJlc3VsdHMucHVzaChcIiAgXCIgKyB0aGlzLnByb3RvYnVmRGVmaW5pdGlvbkZvclByb3AocCwgdW5pdmVyc2FsQmFzZSkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIGN1c3RvbUJhc2UgPSBjdXN0b21CYXNlICsgMTtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXCIgIFwiICsgdGhpcy5wcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wKHAsIGN1c3RvbUJhc2UpKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgICAgcmVzdWx0cy5wdXNoKFwifVwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHByb3RvYnVmSW1wb3J0cygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTsgLy8gVGhpcyBjcmVhdGVzIGEgbmV3bGluZSFcbiAgICBjb25zdCByZXN1bHRTZXQgPSB0aGlzLnByb3RvYnVmSW1wb3J0V2FsayhcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0cy5tYXAodiA9PiB2LnJvb3RQcm9wKSxcbiAgICApO1xuICAgIGZvciAoY29uc3QgbGluZSBvZiByZXN1bHRTZXQudmFsdWVzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChgaW1wb3J0IFwiJHtsaW5lfVwiO2ApO1xuICAgIH1cbiAgICBpZiAocmVzdWx0cy5sZW5ndGggPiAwKSB7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCIvLyBObyBJbXBvcnRzXCI7XG4gICAgfVxuICB9XG5cbiAgcHJvdG9idWZJbXBvcnRXYWxrKHByb3BzOiBQcm9wc1tdKTogU2V0PHN0cmluZz4ge1xuICAgIGNvbnN0IHJlc3VsdDogU2V0PHN0cmluZz4gPSBuZXcgU2V0KCk7XG5cbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcHMpIHtcbiAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcImxpbmtcIikge1xuICAgICAgICBjb25zdCBpbXBvcnRQYXRoID0gdGhpcy5wcm90b2J1ZkltcG9ydEZvclByb3AocHJvcCk7XG4gICAgICAgIGlmIChcbiAgICAgICAgICBpbXBvcnRQYXRoICYmXG4gICAgICAgICAgIWltcG9ydFBhdGguc3RhcnRzV2l0aChcbiAgICAgICAgICAgIGBzaS1yZWdpc3RyeS9wcm90by8ke3RoaXMucHJvdG9idWZQYWNrYWdlTmFtZSgpfWAsXG4gICAgICAgICAgKVxuICAgICAgICApIHtcbiAgICAgICAgICByZXN1bHQuYWRkKGltcG9ydFBhdGgpO1xuICAgICAgICB9XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHQuYWRkKFwiZ29vZ2xlL3Byb3RvYnVmL3dyYXBwZXJzLnByb3RvXCIpO1xuICAgICAgfVxuXG4gICAgICBpZiAodGhpcy5yZWN1cnNlS2luZHMuaW5jbHVkZXMocHJvcC5raW5kKCkpKSB7XG4gICAgICAgIGZvciAoY29uc3QgYmFnIG9mIHByb3AuYmFnTmFtZXMoKSkge1xuICAgICAgICAgIGZvciAoY29uc3QgaW50ZXJuYWxQcm9wIG9mIHByb3BbYmFnXS5hdHRycykge1xuICAgICAgICAgICAgY29uc3QgbmV3U2V0ID0gdGhpcy5wcm90b2J1ZkltcG9ydFdhbGsoW2ludGVybmFsUHJvcF0pO1xuICAgICAgICAgICAgZm9yIChjb25zdCBpdGVtIG9mIG5ld1NldC52YWx1ZXMoKSkge1xuICAgICAgICAgICAgICByZXN1bHQuYWRkKGl0ZW0pO1xuICAgICAgICAgICAgfVxuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgZ2VuZXJhdGVTdHJpbmcoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3Byb3RvYnVmL3Byb3RvJywgeyBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiB0aGlzLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IF9fZmlsZW5hbWUsXG4gICAgICB9LFxuICAgICk7XG4gIH1cbn1cblxuLy9leHBvcnQgY2xhc3MgQ29kZWdlblByb3RvYnVmIHtcbi8vICBjb21wb25lbnQ6IENvbXBvbmVudDtcbi8vXG4vLyAgY29uc3RydWN0b3IoY29tcG9uZW50OiBDb21wb25lbnQpIHtcbi8vICAgIHRoaXMuY29tcG9uZW50ID0gY29tcG9uZW50O1xuLy8gIH1cbi8vXG4vLyAgZ2VuZXJhdGVTdHJpbmcoKTogc3RyaW5nIHtcbi8vICAgIHJldHVybiBlanMucmVuZGVyKFxuLy8gICAgICBcIjwlLSBpbmNsdWRlKCdwcm90b2J1Zi9mdWxsJywgeyBjb21wb25lbnQ6IGNvbXBvbmVudCB9KSAlPlwiLFxuLy8gICAgICB7XG4vLyAgICAgICAgY29tcG9uZW50OiB0aGlzLmNvbXBvbmVudCxcbi8vICAgICAgfSxcbi8vICAgICAge1xuLy8gICAgICAgIGZpbGVuYW1lOiBfX2ZpbGVuYW1lLFxuLy8gICAgICB9LFxuLy8gICAgKTtcbi8vICB9XG4vL31cbiJdfQ==