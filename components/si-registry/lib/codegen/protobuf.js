"use strict";

var _interopRequireWildcard = require("@babel/runtime/helpers/interopRequireWildcard");

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.ProtobufFormatter = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _ejs = _interopRequireDefault(require("ejs"));

var _attrList = require("../attrList");

var _enum = require("../prop/enum");

var PropPrelude = _interopRequireWildcard(require("../components/prelude"));

var _changeCase = require("change-case");

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(n); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

var ProtobufFormatter = /*#__PURE__*/function () {
  function ProtobufFormatter(systemObjects) {
    (0, _classCallCheck2["default"])(this, ProtobufFormatter);
    (0, _defineProperty2["default"])(this, "systemObjects", void 0);
    (0, _defineProperty2["default"])(this, "recurseKinds", ["object"]);

    if (systemObjects.length == 0) {
      throw "You must provide objects to generate";
    }

    this.systemObjects = systemObjects;
  }

  (0, _createClass2["default"])(ProtobufFormatter, [{
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
        } else if (prop.numberKind == "u128") {
          return "google.protobuf.StringValue";
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

      return "unreachable!";
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

          // @ts-ignore
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
          var customBase = 1000; // @ts-ignore

          for (var index in prop[bag].attrs) {
            // @ts-ignore
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

                // @ts-ignore
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
      return _ejs["default"].render("<%- include('src/codegen/protobuf/proto', { fmt }) %>", {
        fmt: this
      }, {
        filename: "."
      });
    }
  }]);
  return ProtobufFormatter;
}();

exports.ProtobufFormatter = ProtobufFormatter;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3Byb3RvYnVmLnRzIl0sIm5hbWVzIjpbIlByb3RvYnVmRm9ybWF0dGVyIiwic3lzdGVtT2JqZWN0cyIsImxlbmd0aCIsImZpcnN0Iiwic2VydmljZU5hbWUiLCJyZXN1bHRzIiwiZmlsdGVyIiwib2JqIiwibWV0aG9kc1Byb3AiLCJwcm9wZXJ0aWVzIiwicHVzaCIsIm9iamVjdCIsIm1ldGhvZHMiLCJhdHRycyIsIm1ldGhvZCIsIm1ldGhvZE5hbWUiLCJwYXJlbnROYW1lIiwibmFtZSIsImpvaW4iLCJwcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0Iiwicm9vdFByb3AiLCJtZXRob2RIb2xkZXIiLCJQcm9wUHJlbHVkZSIsIlByb3BNZXRob2QiLCJQcm9wQWN0aW9uIiwicmVxdWVzdCIsInJlcGx5IiwidHlwZU5hbWUiLCJwcm9wIiwiUHJvcExpbmsiLCJwcm9wT3duZXIiLCJsb29rdXBPYmplY3QiLCJwYXRoTmFtZSIsIlByb3BCb29sIiwiUHJvcENvZGUiLCJQcm9wRW51bSIsInJlYWxQcm9wIiwibG9va3VwTXlzZWxmIiwiUHJvcE9iamVjdCIsInByb3RvYnVmVHlwZUZvclByb3AiLCJQcm9wTWFwIiwiUHJvcE51bWJlciIsIm51bWJlcktpbmQiLCJwcm90b2J1ZlBhY2thZ2VOYW1lIiwiUHJvcFNlbGVjdCIsIlByb3BUZXh0Iiwia2luZCIsImlucHV0TnVtYmVyIiwicmVwZWF0ZWQiLCJlbnVtQ291bnQiLCJ2YXJpYW50cyIsInZhcmlhbnQiLCJiYWdOYW1lcyIsImJhZyIsImNoaWxkUHJvcCIsIm1lc3NhZ2VOYW1lIiwidW5pdmVyc2FsQmFzZSIsImN1c3RvbUJhc2UiLCJpbmRleCIsInAiLCJ1bml2ZXJzYWwiLCJwcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wIiwicmVzdWx0U2V0IiwicHJvdG9idWZJbXBvcnRXYWxrIiwibWFwIiwidiIsInZhbHVlcyIsImxpbmUiLCJwcm9wcyIsInJlc3VsdCIsIlNldCIsImltcG9ydFBhdGgiLCJwcm90b2J1ZkltcG9ydEZvclByb3AiLCJzdGFydHNXaXRoIiwiYWRkIiwicmVjdXJzZUtpbmRzIiwiaW5jbHVkZXMiLCJpbnRlcm5hbFByb3AiLCJuZXdTZXQiLCJpdGVtIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBRUE7Ozs7Ozs7O0lBRWFBLGlCO0FBS1gsNkJBQVlDLGFBQVosRUFBMEM7QUFBQTtBQUFBO0FBQUEsMkRBRjNCLENBQUMsUUFBRCxDQUUyQjs7QUFDeEMsUUFBSUEsYUFBYSxDQUFDQyxNQUFkLElBQXdCLENBQTVCLEVBQStCO0FBQzdCLFlBQU0sc0NBQU47QUFDRDs7QUFDRCxTQUFLRCxhQUFMLEdBQXFCQSxhQUFyQjtBQUNEOzs7OzRCQUVvQjtBQUNuQixhQUFPLEtBQUtBLGFBQUwsQ0FBbUIsQ0FBbkIsQ0FBUDtBQUNEOzs7MENBRTZCO0FBQzVCLDBCQUFhLDJCQUFVLEtBQUtFLEtBQUwsR0FBYUMsV0FBdkIsQ0FBYjtBQUNEOzs7dUNBRTBCO0FBQ3pCLFVBQU1DLE9BQU8sR0FBRyxFQUFoQjs7QUFDQSxVQUNFLEtBQUtKLGFBQUwsQ0FBbUJLLE1BQW5CLENBQTBCLFVBQUFDLEdBQUc7QUFBQSxlQUFJQSxHQUFHLENBQUNDLFdBQUosQ0FBZ0JDLFVBQWhCLENBQTJCUCxNQUEzQixHQUFvQyxDQUF4QztBQUFBLE9BQTdCLEVBQ0dBLE1BREgsR0FDWSxDQUZkLEVBR0U7QUFDQUcsUUFBQUEsT0FBTyxDQUFDSyxJQUFSLG1CQUF3Qiw0QkFBVyxLQUFLUCxLQUFMLEdBQWFDLFdBQXhCLENBQXhCOztBQURBLG1EQUVxQixLQUFLSCxhQUYxQjtBQUFBOztBQUFBO0FBRUEsOERBQXlDO0FBQUEsZ0JBQTlCVSxNQUE4Qjs7QUFBQSx3REFDbEJBLE1BQU0sQ0FBQ0MsT0FBUCxDQUFlQyxLQURHO0FBQUE7O0FBQUE7QUFDdkMscUVBQTJDO0FBQUEsb0JBQWhD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L3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IGVqcyBmcm9tIFwiZWpzXCI7XG5pbXBvcnQgeyBQcm9wcywgUHJvcE9iamVjdCB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuaW1wb3J0IHsgUHJvcEVudW0gfSBmcm9tIFwiLi4vcHJvcC9lbnVtXCI7XG5pbXBvcnQgKiBhcyBQcm9wUHJlbHVkZSBmcm9tIFwiLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5cbmltcG9ydCB7IHNuYWtlQ2FzZSwgcGFzY2FsQ2FzZSwgY29uc3RhbnRDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5cbmV4cG9ydCBjbGFzcyBQcm90b2J1ZkZvcm1hdHRlciB7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgcmVjdXJzZUtpbmRzID0gW1wib2JqZWN0XCJdO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW10pIHtcbiAgICBpZiAoc3lzdGVtT2JqZWN0cy5sZW5ndGggPT0gMCkge1xuICAgICAgdGhyb3cgXCJZb3UgbXVzdCBwcm92aWRlIG9iamVjdHMgdG8gZ2VuZXJhdGVcIjtcbiAgICB9XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gc3lzdGVtT2JqZWN0cztcbiAgfVxuXG4gIGZpcnN0KCk6IE9iamVjdFR5cGVzIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzWzBdO1xuICB9XG5cbiAgcHJvdG9idWZQYWNrYWdlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgc2kuJHtzbmFrZUNhc2UodGhpcy5maXJzdCgpLnNlcnZpY2VOYW1lKX1gO1xuICB9XG5cbiAgcHJvdG9idWZTZXJ2aWNlcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdHMuZmlsdGVyKG9iaiA9PiBvYmoubWV0aG9kc1Byb3AucHJvcGVydGllcy5sZW5ndGggPiAwKVxuICAgICAgICAubGVuZ3RoID4gMFxuICAgICkge1xuICAgICAgcmVzdWx0cy5wdXNoKGBzZXJ2aWNlICR7cGFzY2FsQ2FzZSh0aGlzLmZpcnN0KCkuc2VydmljZU5hbWUpfSB7YCk7XG4gICAgICBmb3IgKGNvbnN0IG9iamVjdCBvZiB0aGlzLnN5c3RlbU9iamVjdHMpIHtcbiAgICAgICAgZm9yIChjb25zdCBtZXRob2Qgb2Ygb2JqZWN0Lm1ldGhvZHMuYXR0cnMpIHtcbiAgICAgICAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgICAgICAgIHBhc2NhbENhc2UobWV0aG9kLnBhcmVudE5hbWUpICsgcGFzY2FsQ2FzZShtZXRob2QubmFtZSk7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICAgICAgYCAgcnBjICR7bWV0aG9kTmFtZX0oJHttZXRob2ROYW1lfVJlcXVlc3QpIHJldHVybnMgKCR7bWV0aG9kTmFtZX1SZXBseSk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgICByZXN1bHRzLnB1c2goYH1gKTtcbiAgICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gICAgfVxuICAgIHJldHVybiBcIi8vIE5vIFNlcnZpY2VzXCI7XG4gIH1cblxuICBwcm90b2J1Zk1lc3NhZ2VzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGZvciAoY29uc3Qgb2JqZWN0IG9mIHRoaXMuc3lzdGVtT2JqZWN0cykge1xuICAgICAgcmVzdWx0cy5wdXNoKHRoaXMucHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChvYmplY3Qucm9vdFByb3ApKTtcbiAgICAgIGlmIChvYmplY3QubWV0aG9kc1Byb3AucHJvcGVydGllcy5sZW5ndGgpIHtcbiAgICAgICAgZm9yIChjb25zdCBtZXRob2RIb2xkZXIgb2Ygb2JqZWN0Lm1ldGhvZHNQcm9wLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgICBpZiAoXG4gICAgICAgICAgICBtZXRob2RIb2xkZXIgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHx8XG4gICAgICAgICAgICBtZXRob2RIb2xkZXIgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uXG4gICAgICAgICAgKSB7XG4gICAgICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgICAgIHRoaXMucHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChtZXRob2RIb2xkZXIucmVxdWVzdCksXG4gICAgICAgICAgICApO1xuICAgICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMucHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChtZXRob2RIb2xkZXIucmVwbHkpKTtcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgdGhyb3cgYEVycm9yIGdlbmVyYXRpbmcgcHJvdG9idWYgLSBub24gbWV0aG9kL2FjdGlvbiBwcm9wIGZvdW5kIG9uICR7b2JqZWN0LnR5cGVOYW1lfWA7XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBwcm90b2J1ZkltcG9ydEZvclByb3AocHJvcDogUHJvcHMpOiBzdHJpbmcge1xuICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICBsZXQgcGF0aE5hbWUgPSBcInNpLXJlZ2lzdHJ5L3Byb3RvL3NpLlwiO1xuICAgICAgaWYgKHByb3BPd25lci5zZXJ2aWNlTmFtZSkge1xuICAgICAgICBwYXRoTmFtZSA9IHBhdGhOYW1lICsgc25ha2VDYXNlKHByb3BPd25lci5zZXJ2aWNlTmFtZSkgKyBcIi5wcm90b1wiO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcGF0aE5hbWUgPSBwYXRoTmFtZSArIHNuYWtlQ2FzZShwcm9wT3duZXIudHlwZU5hbWUpICsgXCIucHJvdG9cIjtcbiAgICAgIH1cbiAgICAgIHJldHVybiBwYXRoTmFtZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9XG5cbiAgcHJvdG9idWZUeXBlRm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQm9vbCkge1xuICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLkJvb2xWYWx1ZVwiO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlKSB7XG4gICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuU3RyaW5nVmFsdWVcIjtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wRW51bSkge1xuICAgICAgcmV0dXJuIGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICBpZiAoXG4gICAgICAgIHJlYWxQcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCB8fFxuICAgICAgICByZWFsUHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BFbnVtXG4gICAgICApIHtcbiAgICAgICAgY29uc3QgcHJvcE93bmVyID0gcHJvcC5sb29rdXBPYmplY3QoKTtcbiAgICAgICAgbGV0IHBhdGhOYW1lID0gXCJzaS5cIjtcbiAgICAgICAgaWYgKHByb3BPd25lci5zZXJ2aWNlTmFtZSkge1xuICAgICAgICAgIHBhdGhOYW1lID0gcGF0aE5hbWUgKyBzbmFrZUNhc2UocHJvcE93bmVyLnNlcnZpY2VOYW1lKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IHBhdGhOYW1lICsgc25ha2VDYXNlKHByb3BPd25lci50eXBlTmFtZSk7XG4gICAgICAgIH1cbiAgICAgICAgcmV0dXJuIGAke3BhdGhOYW1lfS4ke3Bhc2NhbENhc2UocmVhbFByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICAgIHJlYWxQcm9wLm5hbWUsXG4gICAgICAgICl9YDtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiB0aGlzLnByb3RvYnVmVHlwZUZvclByb3AocmVhbFByb3ApO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNYXApIHtcbiAgICAgIHJldHVybiBcIm1hcDxzdHJpbmcsIHN0cmluZz5cIjtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTnVtYmVyKSB7XG4gICAgICBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50MzJcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuSW50MzJWYWx1ZVwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1aW50MzJcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuVUludDMyVmFsdWVcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50NjRcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuSW50NjRWYWx1ZVwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1aW50NjRcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuVUludDY0VmFsdWVcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidTEyOFwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5TdHJpbmdWYWx1ZVwiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgIHJldHVybiBgJHt0aGlzLnByb3RvYnVmUGFja2FnZU5hbWUoKX0uJHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLnBhcmVudE5hbWUsXG4gICAgICApfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgcmV0dXJuIGAke3RoaXMucHJvdG9idWZQYWNrYWdlTmFtZSgpfS4ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AucGFyZW50TmFtZSxcbiAgICAgICl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKSB7XG4gICAgICByZXR1cm4gYCR7dGhpcy5wcm90b2J1ZlBhY2thZ2VOYW1lKCl9LiR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5wYXJlbnROYW1lLFxuICAgICAgKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFNlbGVjdCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BUZXh0XG4gICAgKSB7XG4gICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuU3RyaW5nVmFsdWVcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgdGhyb3cgYFVua25vd24gcHJvcGVydHkgdHlwZSBmb3IgcmVuZGVyaW5nIHByb3RvYnVmISBGaXggbWU6ICR7cHJvcC5raW5kKCl9YDtcbiAgICB9XG4gICAgcmV0dXJuIFwidW5yZWFjaGFibGUhXCI7XG4gIH1cblxuICBwcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wKHByb3A6IFByb3BzLCBpbnB1dE51bWJlcjogbnVtYmVyKTogc3RyaW5nIHtcbiAgICBsZXQgcmVwZWF0ZWQ6IHN0cmluZztcbiAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgcmVwZWF0ZWQgPSBcInJlcGVhdGVkIFwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXBlYXRlZCA9IFwiXCI7XG4gICAgfVxuICAgIHJldHVybiBgJHtyZXBlYXRlZH0ke3RoaXMucHJvdG9idWZUeXBlRm9yUHJvcChwcm9wKX0gJHtzbmFrZUNhc2UoXG4gICAgICBwcm9wLm5hbWUsXG4gICAgKX0gPSAke2lucHV0TnVtYmVyfTtgO1xuICB9XG5cbiAgcHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChwcm9wOiBQcm9wT2JqZWN0IHwgUHJvcEVudW0pOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcblxuICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcEVudW0pIHtcbiAgICAgIGxldCBlbnVtQ291bnQgPSAwO1xuICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICBgZW51bSAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX0ge2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICBgICAke2NvbnN0YW50Q2FzZShcbiAgICAgICAgICB0aGlzLnByb3RvYnVmVHlwZUZvclByb3AocHJvcCksXG4gICAgICAgICl9X1VOS05PV04gPSAke2VudW1Db3VudH07YCxcbiAgICAgICk7XG4gICAgICBmb3IgKGNvbnN0IHZhcmlhbnQgb2YgcHJvcC52YXJpYW50cykge1xuICAgICAgICBlbnVtQ291bnQgPSBlbnVtQ291bnQgKyAxO1xuICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgYCAgJHtjb25zdGFudENhc2UodGhpcy5wcm90b2J1ZlR5cGVGb3JQcm9wKHByb3ApKX1fJHtjb25zdGFudENhc2UoXG4gICAgICAgICAgICB2YXJpYW50LFxuICAgICAgICAgICl9ID0gJHtlbnVtQ291bnR9O2AsXG4gICAgICAgICk7XG4gICAgICB9XG4gICAgICByZXN1bHRzLnB1c2goXCJ9XCIpO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9XG5cbiAgICBmb3IgKGNvbnN0IGJhZyBvZiBwcm9wLmJhZ05hbWVzKCkpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIGZvciAoY29uc3QgY2hpbGRQcm9wIG9mIHByb3BbYmFnXS5hdHRycykge1xuICAgICAgICBpZiAoY2hpbGRQcm9wIGluc3RhbmNlb2YgUHJvcE9iamVjdCB8fCBjaGlsZFByb3AgaW5zdGFuY2VvZiBQcm9wRW51bSkge1xuICAgICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLnByb3RvYnVmTWVzc2FnZUZvclByb3BPYmplY3QoY2hpbGRQcm9wKSk7XG4gICAgICAgIH1cbiAgICAgIH1cblxuICAgICAgY29uc3QgbWVzc2FnZU5hbWUgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLm5hbWUsXG4gICAgICApfWA7XG4gICAgICByZXN1bHRzLnB1c2goYG1lc3NhZ2UgJHttZXNzYWdlTmFtZX0ge2ApO1xuXG4gICAgICBsZXQgdW5pdmVyc2FsQmFzZSA9IDA7XG4gICAgICBsZXQgY3VzdG9tQmFzZSA9IDEwMDA7XG4gICAgICAvLyBAdHMtaWdub3JlXG4gICAgICBmb3IgKGNvbnN0IGluZGV4IGluIHByb3BbYmFnXS5hdHRycykge1xuICAgICAgICAvLyBAdHMtaWdub3JlXG4gICAgICAgIGNvbnN0IHAgPSBwcm9wW2JhZ10uYXR0cnNbaW5kZXhdO1xuXG4gICAgICAgIGlmIChwLnVuaXZlcnNhbCkge1xuICAgICAgICAgIHVuaXZlcnNhbEJhc2UgPSB1bml2ZXJzYWxCYXNlICsgMTtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXCIgIFwiICsgdGhpcy5wcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wKHAsIHVuaXZlcnNhbEJhc2UpKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBjdXN0b21CYXNlID0gY3VzdG9tQmFzZSArIDE7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKFwiICBcIiArIHRoaXMucHJvdG9idWZEZWZpbml0aW9uRm9yUHJvcChwLCBjdXN0b21CYXNlKSk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICAgIHJlc3VsdHMucHVzaChcIn1cIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBwcm90b2J1ZkltcG9ydHMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107IC8vIFRoaXMgY3JlYXRlcyBhIG5ld2xpbmUhXG4gICAgY29uc3QgcmVzdWx0U2V0ID0gdGhpcy5wcm90b2J1ZkltcG9ydFdhbGsoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdHMubWFwKHYgPT4gdi5yb290UHJvcCksXG4gICAgKTtcbiAgICBmb3IgKGNvbnN0IGxpbmUgb2YgcmVzdWx0U2V0LnZhbHVlcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goYGltcG9ydCBcIiR7bGluZX1cIjtgKTtcbiAgICB9XG4gICAgaWYgKHJlc3VsdHMubGVuZ3RoID4gMCkge1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiLy8gTm8gSW1wb3J0c1wiO1xuICAgIH1cbiAgfVxuXG4gIHByb3RvYnVmSW1wb3J0V2Fsayhwcm9wczogUHJvcHNbXSk6IFNldDxzdHJpbmc+IHtcbiAgICBjb25zdCByZXN1bHQ6IFNldDxzdHJpbmc+ID0gbmV3IFNldCgpO1xuXG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHByb3BzKSB7XG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgICAgY29uc3QgaW1wb3J0UGF0aCA9IHRoaXMucHJvdG9idWZJbXBvcnRGb3JQcm9wKHByb3ApO1xuICAgICAgICBpZiAoXG4gICAgICAgICAgaW1wb3J0UGF0aCAmJlxuICAgICAgICAgICFpbXBvcnRQYXRoLnN0YXJ0c1dpdGgoXG4gICAgICAgICAgICBgc2ktcmVnaXN0cnkvcHJvdG8vJHt0aGlzLnByb3RvYnVmUGFja2FnZU5hbWUoKX1gLFxuICAgICAgICAgIClcbiAgICAgICAgKSB7XG4gICAgICAgICAgcmVzdWx0LmFkZChpbXBvcnRQYXRoKTtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0LmFkZChcImdvb2dsZS9wcm90b2J1Zi93cmFwcGVycy5wcm90b1wiKTtcbiAgICAgIH1cblxuICAgICAgaWYgKHRoaXMucmVjdXJzZUtpbmRzLmluY2x1ZGVzKHByb3Aua2luZCgpKSkge1xuICAgICAgICBmb3IgKGNvbnN0IGJhZyBvZiBwcm9wLmJhZ05hbWVzKCkpIHtcbiAgICAgICAgICAvLyBAdHMtaWdub3JlXG4gICAgICAgICAgZm9yIChjb25zdCBpbnRlcm5hbFByb3Agb2YgcHJvcFtiYWddLmF0dHJzKSB7XG4gICAgICAgICAgICBjb25zdCBuZXdTZXQgPSB0aGlzLnByb3RvYnVmSW1wb3J0V2FsayhbaW50ZXJuYWxQcm9wXSk7XG4gICAgICAgICAgICBmb3IgKGNvbnN0IGl0ZW0gb2YgbmV3U2V0LnZhbHVlcygpKSB7XG4gICAgICAgICAgICAgIHJlc3VsdC5hZGQoaXRlbSk7XG4gICAgICAgICAgICB9XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQ7XG4gIH1cblxuICBnZW5lcmF0ZVN0cmluZygpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcHJvdG9idWYvcHJvdG8nLCB7IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IHRoaXMsXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gIH1cbn1cbiJdfQ==