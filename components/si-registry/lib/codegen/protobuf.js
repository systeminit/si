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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3Byb3RvYnVmLnRzIl0sIm5hbWVzIjpbIlByb3RvYnVmRm9ybWF0dGVyIiwic3lzdGVtT2JqZWN0cyIsImxlbmd0aCIsImZpcnN0Iiwic2VydmljZU5hbWUiLCJyZXN1bHRzIiwiZmlsdGVyIiwib2JqIiwibWV0aG9kc1Byb3AiLCJwcm9wZXJ0aWVzIiwicHVzaCIsIm9iamVjdCIsIm1ldGhvZHMiLCJhdHRycyIsIm1ldGhvZCIsIm1ldGhvZE5hbWUiLCJwYXJlbnROYW1lIiwibmFtZSIsImpvaW4iLCJwcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0Iiwicm9vdFByb3AiLCJtZXRob2RIb2xkZXIiLCJQcm9wUHJlbHVkZSIsIlByb3BNZXRob2QiLCJQcm9wQWN0aW9uIiwicmVxdWVzdCIsInJlcGx5IiwidHlwZU5hbWUiLCJwcm9wIiwiUHJvcExpbmsiLCJwcm9wT3duZXIiLCJsb29rdXBPYmplY3QiLCJwYXRoTmFtZSIsIlByb3BCb29sIiwiUHJvcENvZGUiLCJQcm9wRW51bSIsInJlYWxQcm9wIiwibG9va3VwTXlzZWxmIiwiUHJvcE9iamVjdCIsInByb3RvYnVmVHlwZUZvclByb3AiLCJQcm9wTWFwIiwiUHJvcE51bWJlciIsIm51bWJlcktpbmQiLCJwcm90b2J1ZlBhY2thZ2VOYW1lIiwiUHJvcFNlbGVjdCIsIlByb3BUZXh0Iiwia2luZCIsImlucHV0TnVtYmVyIiwicmVwZWF0ZWQiLCJlbnVtQ291bnQiLCJ2YXJpYW50cyIsInZhcmlhbnQiLCJiYWdOYW1lcyIsImJhZyIsImNoaWxkUHJvcCIsIm1lc3NhZ2VOYW1lIiwidW5pdmVyc2FsQmFzZSIsImN1c3RvbUJhc2UiLCJpbmRleCIsInAiLCJ1bml2ZXJzYWwiLCJwcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wIiwicmVzdWx0U2V0IiwicHJvdG9idWZJbXBvcnRXYWxrIiwibWFwIiwidiIsInZhbHVlcyIsImxpbmUiLCJwcm9wcyIsInJlc3VsdCIsIlNldCIsImltcG9ydFBhdGgiLCJwcm90b2J1ZkltcG9ydEZvclByb3AiLCJzdGFydHNXaXRoIiwiYWRkIiwicmVjdXJzZUtpbmRzIiwiaW5jbHVkZXMiLCJpbnRlcm5hbFByb3AiLCJuZXdTZXQiLCJpdGVtIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBRUE7Ozs7Ozs7O0lBRWFBLGlCO0FBS1gsNkJBQVlDLGFBQVosRUFBMEM7QUFBQTtBQUFBO0FBQUEsMkRBRjNCLENBQUMsUUFBRCxDQUUyQjs7QUFDeEMsUUFBSUEsYUFBYSxDQUFDQyxNQUFkLElBQXdCLENBQTVCLEVBQStCO0FBQzdCLFlBQU0sc0NBQU47QUFDRDs7QUFDRCxTQUFLRCxhQUFMLEdBQXFCQSxhQUFyQjtBQUNEOzs7OzRCQUVvQjtBQUNuQixhQUFPLEtBQUtBLGFBQUwsQ0FBbUIsQ0FBbkIsQ0FBUDtBQUNEOzs7MENBRTZCO0FBQzVCLDBCQUFhLDJCQUFVLEtBQUtFLEtBQUwsR0FBYUMsV0FBdkIsQ0FBYjtBQUNEOzs7dUNBRTBCO0FBQ3pCLFVBQU1DLE9BQU8sR0FBRyxFQUFoQjs7QUFDQSxVQUNFLEtBQUtKLGFBQUwsQ0FBbUJLLE1BQW5CLENBQTBCLFVBQUFDLEdBQUc7QUFBQSxlQUFJQSxHQUFHLENBQUNDLFdBQUosQ0FBZ0JDLFVBQWhCLENBQTJCUCxNQUEzQixHQUFvQyxDQUF4QztBQUFBLE9BQTdCLEVBQ0dBLE1BREgsR0FDWSxDQUZkLEVBR0U7QUFDQUcsUUFBQUEsT0FBTyxDQUFDSyxJQUFSLG1CQUF3Qiw0QkFBVyxLQUFLUCxLQUFMLEdBQWFDLFdBQXhCLENBQXhCOztBQURBLG1EQUVxQixLQUFLSCxhQUYxQjtBQUFBOztBQUFBO0FBRUEsOERBQXlDO0FBQUEsZ0JBQTlCVSxNQUE4Qjs7QUFBQSx3REFDbEJBLE1BQU0sQ0FBQ0MsT0FBUCxDQUFlQyxLQURHO0FBQUE7O0FBQUE7QUFDdkMscUVBQTJDO0FBQUEsb0JBQWhD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LE1BVU87QUFDTEYsWUFBQUEsTUFBTSxDQUFDSyxHQUFQLENBQVcsZ0NBQVg7QUFDRDs7QUFFRCxjQUFJLEtBQUtDLFlBQUwsQ0FBa0JDLFFBQWxCLENBQTJCaEQsSUFBSSxDQUFDa0IsSUFBTCxFQUEzQixDQUFKLEVBQTZDO0FBQUEseURBQ3pCbEIsSUFBSSxDQUFDd0IsUUFBTCxFQUR5QjtBQUFBOztBQUFBO0FBQzNDLHdFQUFtQztBQUFBLG9CQUF4QkMsR0FBd0I7O0FBQ2pDO0FBRGlDLDZEQUVOekIsSUFBSSxDQUFDeUIsR0FBRCxDQUFKLENBQVV4QyxLQUZKO0FBQUE7O0FBQUE7QUFFakMsNEVBQTRDO0FBQUEsd0JBQWpDZ0UsWUFBaUM7QUFDMUMsd0JBQU1DLE1BQU0sR0FBRyxLQUFLZixrQkFBTCxDQUF3QixDQUFDYyxZQUFELENBQXhCLENBQWY7O0FBRDBDLGlFQUV2QkMsTUFBTSxDQUFDWixNQUFQLEVBRnVCO0FBQUE7O0FBQUE7QUFFMUMsZ0ZBQW9DO0FBQUEsNEJBQXpCYSxJQUF5QjtBQUNsQ1Ysd0JBQUFBLE1BQU0sQ0FBQ0ssR0FBUCxDQUFXSyxJQUFYO0FBQ0Q7QUFKeUM7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUszQztBQVBnQztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBUWxDO0FBVDBDO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFVNUM7QUFDRjtBQTdCNkM7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUE4QjlDLGFBQU9WLE1BQVA7QUFDRDs7O3FDQUV3QjtBQUN2QixhQUFPVyxnQkFBSUMsTUFBSixDQUNMLHVEQURLLEVBRUw7QUFDRUMsUUFBQUEsR0FBRyxFQUFFO0FBRFAsT0FGSyxFQUtMO0FBQ0VDLFFBQUFBLFFBQVEsRUFBRTtBQURaLE9BTEssQ0FBUDtBQVNEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgZWpzIGZyb20gXCJlanNcIjtcbmltcG9ydCB7IFByb3BzLCBQcm9wT2JqZWN0IH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5pbXBvcnQgeyBQcm9wRW51bSB9IGZyb20gXCIuLi9wcm9wL2VudW1cIjtcbmltcG9ydCAqIGFzIFByb3BQcmVsdWRlIGZyb20gXCIuLi9jb21wb25lbnRzL3ByZWx1ZGVcIjtcblxuaW1wb3J0IHsgc25ha2VDYXNlLCBwYXNjYWxDYXNlLCBjb25zdGFudENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcblxuZXhwb3J0IGNsYXNzIFByb3RvYnVmRm9ybWF0dGVyIHtcbiAgc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICByZWN1cnNlS2luZHMgPSBbXCJvYmplY3RcIl07XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXSkge1xuICAgIGlmIChzeXN0ZW1PYmplY3RzLmxlbmd0aCA9PSAwKSB7XG4gICAgICB0aHJvdyBcIllvdSBtdXN0IHByb3ZpZGUgb2JqZWN0cyB0byBnZW5lcmF0ZVwiO1xuICAgIH1cbiAgICB0aGlzLnN5c3RlbU9iamVjdHMgPSBzeXN0ZW1PYmplY3RzO1xuICB9XG5cbiAgZmlyc3QoKTogT2JqZWN0VHlwZXMge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHNbMF07XG4gIH1cblxuICBwcm90b2J1ZlBhY2thZ2VOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBzaS4ke3NuYWtlQ2FzZSh0aGlzLmZpcnN0KCkuc2VydmljZU5hbWUpfWA7XG4gIH1cblxuICBwcm90b2J1ZlNlcnZpY2VzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0cy5maWx0ZXIob2JqID0+IG9iai5tZXRob2RzUHJvcC5wcm9wZXJ0aWVzLmxlbmd0aCA+IDApXG4gICAgICAgIC5sZW5ndGggPiAwXG4gICAgKSB7XG4gICAgICByZXN1bHRzLnB1c2goYHNlcnZpY2UgJHtwYXNjYWxDYXNlKHRoaXMuZmlyc3QoKS5zZXJ2aWNlTmFtZSl9IHtgKTtcbiAgICAgIGZvciAoY29uc3Qgb2JqZWN0IG9mIHRoaXMuc3lzdGVtT2JqZWN0cykge1xuICAgICAgICBmb3IgKGNvbnN0IG1ldGhvZCBvZiBvYmplY3QubWV0aG9kcy5hdHRycykge1xuICAgICAgICAgIGNvbnN0IG1ldGhvZE5hbWUgPVxuICAgICAgICAgICAgcGFzY2FsQ2FzZShtZXRob2QucGFyZW50TmFtZSkgKyBwYXNjYWxDYXNlKG1ldGhvZC5uYW1lKTtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgICBgICBycGMgJHttZXRob2ROYW1lfSgke21ldGhvZE5hbWV9UmVxdWVzdCkgcmV0dXJucyAoJHttZXRob2ROYW1lfVJlcGx5KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICAgIHJlc3VsdHMucHVzaChgfWApO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9XG4gICAgcmV0dXJuIFwiLy8gTm8gU2VydmljZXNcIjtcbiAgfVxuXG4gIHByb3RvYnVmTWVzc2FnZXMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgZm9yIChjb25zdCBvYmplY3Qgb2YgdGhpcy5zeXN0ZW1PYmplY3RzKSB7XG4gICAgICByZXN1bHRzLnB1c2godGhpcy5wcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KG9iamVjdC5yb290UHJvcCkpO1xuICAgICAgaWYgKG9iamVjdC5tZXRob2RzUHJvcC5wcm9wZXJ0aWVzLmxlbmd0aCkge1xuICAgICAgICBmb3IgKGNvbnN0IG1ldGhvZEhvbGRlciBvZiBvYmplY3QubWV0aG9kc1Byb3AucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICAgIGlmIChcbiAgICAgICAgICAgIG1ldGhvZEhvbGRlciBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QgfHxcbiAgICAgICAgICAgIG1ldGhvZEhvbGRlciBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb25cbiAgICAgICAgICApIHtcbiAgICAgICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICAgICAgdGhpcy5wcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KG1ldGhvZEhvbGRlci5yZXF1ZXN0KSxcbiAgICAgICAgICAgICk7XG4gICAgICAgICAgICByZXN1bHRzLnB1c2godGhpcy5wcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KG1ldGhvZEhvbGRlci5yZXBseSkpO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICB0aHJvdyBgRXJyb3IgZ2VuZXJhdGluZyBwcm90b2J1ZiAtIG5vbiBtZXRob2QvYWN0aW9uIHByb3AgZm91bmQgb24gJHtvYmplY3QudHlwZU5hbWV9YDtcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHByb3RvYnVmSW1wb3J0Rm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgY29uc3QgcHJvcE93bmVyID0gcHJvcC5sb29rdXBPYmplY3QoKTtcbiAgICAgIGxldCBwYXRoTmFtZSA9IFwic2ktcmVnaXN0cnkvcHJvdG8vc2kuXCI7XG4gICAgICBpZiAocHJvcE93bmVyLnNlcnZpY2VOYW1lKSB7XG4gICAgICAgIHBhdGhOYW1lID0gcGF0aE5hbWUgKyBzbmFrZUNhc2UocHJvcE93bmVyLnNlcnZpY2VOYW1lKSArIFwiLnByb3RvXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICBwYXRoTmFtZSA9IHBhdGhOYW1lICsgc25ha2VDYXNlKHByb3BPd25lci50eXBlTmFtZSkgKyBcIi5wcm90b1wiO1xuICAgICAgfVxuICAgICAgcmV0dXJuIHBhdGhOYW1lO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJcIjtcbiAgICB9XG4gIH1cblxuICBwcm90b2J1ZlR5cGVGb3JQcm9wKHByb3A6IFByb3BzKTogc3RyaW5nIHtcbiAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BCb29sKSB7XG4gICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuQm9vbFZhbHVlXCI7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcENvZGUpIHtcbiAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5TdHJpbmdWYWx1ZVwiO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BFbnVtKSB7XG4gICAgICByZXR1cm4gYCR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgIGNvbnN0IHJlYWxQcm9wID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIGlmIChcbiAgICAgICAgcmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0IHx8XG4gICAgICAgIHJlYWxQcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEVudW1cbiAgICAgICkge1xuICAgICAgICBjb25zdCBwcm9wT3duZXIgPSBwcm9wLmxvb2t1cE9iamVjdCgpO1xuICAgICAgICBsZXQgcGF0aE5hbWUgPSBcInNpLlwiO1xuICAgICAgICBpZiAocHJvcE93bmVyLnNlcnZpY2VOYW1lKSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBwYXRoTmFtZSArIHNuYWtlQ2FzZShwcm9wT3duZXIuc2VydmljZU5hbWUpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHBhdGhOYW1lID0gcGF0aE5hbWUgKyBzbmFrZUNhc2UocHJvcE93bmVyLnR5cGVOYW1lKTtcbiAgICAgICAgfVxuICAgICAgICByZXR1cm4gYCR7cGF0aE5hbWV9LiR7cGFzY2FsQ2FzZShyZWFsUHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgICAgcmVhbFByb3AubmFtZSxcbiAgICAgICAgKX1gO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHRoaXMucHJvdG9idWZUeXBlRm9yUHJvcChyZWFsUHJvcCk7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1hcCkge1xuICAgICAgcmV0dXJuIFwibWFwPHN0cmluZywgc3RyaW5nPlwiO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BOdW1iZXIpIHtcbiAgICAgIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQzMlwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5JbnQzMlZhbHVlXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQzMlwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5VSW50MzJWYWx1ZVwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQ2NFwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5JbnQ2NFZhbHVlXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQ2NFwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5VSW50NjRWYWx1ZVwiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgIHJldHVybiBgJHt0aGlzLnByb3RvYnVmUGFja2FnZU5hbWUoKX0uJHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLnBhcmVudE5hbWUsXG4gICAgICApfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgcmV0dXJuIGAke3RoaXMucHJvdG9idWZQYWNrYWdlTmFtZSgpfS4ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AucGFyZW50TmFtZSxcbiAgICAgICl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKSB7XG4gICAgICByZXR1cm4gYCR7dGhpcy5wcm90b2J1ZlBhY2thZ2VOYW1lKCl9LiR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5wYXJlbnROYW1lLFxuICAgICAgKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFNlbGVjdCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BUZXh0XG4gICAgKSB7XG4gICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuU3RyaW5nVmFsdWVcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgdGhyb3cgYFVua25vd24gcHJvcGVydHkgdHlwZSBmb3IgcmVuZGVyaW5nIHByb3RvYnVmISBGaXggbWU6ICR7cHJvcC5raW5kKCl9YDtcbiAgICB9XG4gICAgcmV0dXJuIFwidW5yZWFjaGFibGUhXCI7XG4gIH1cblxuICBwcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wKHByb3A6IFByb3BzLCBpbnB1dE51bWJlcjogbnVtYmVyKTogc3RyaW5nIHtcbiAgICBsZXQgcmVwZWF0ZWQ6IHN0cmluZztcbiAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgcmVwZWF0ZWQgPSBcInJlcGVhdGVkIFwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXBlYXRlZCA9IFwiXCI7XG4gICAgfVxuICAgIHJldHVybiBgJHtyZXBlYXRlZH0ke3RoaXMucHJvdG9idWZUeXBlRm9yUHJvcChwcm9wKX0gJHtzbmFrZUNhc2UoXG4gICAgICBwcm9wLm5hbWUsXG4gICAgKX0gPSAke2lucHV0TnVtYmVyfTtgO1xuICB9XG5cbiAgcHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChwcm9wOiBQcm9wT2JqZWN0IHwgUHJvcEVudW0pOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcblxuICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcEVudW0pIHtcbiAgICAgIGxldCBlbnVtQ291bnQgPSAwO1xuICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICBgZW51bSAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX0ge2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICBgICAke2NvbnN0YW50Q2FzZShcbiAgICAgICAgICB0aGlzLnByb3RvYnVmVHlwZUZvclByb3AocHJvcCksXG4gICAgICAgICl9X1VOS05PV04gPSAke2VudW1Db3VudH07YCxcbiAgICAgICk7XG4gICAgICBmb3IgKGNvbnN0IHZhcmlhbnQgb2YgcHJvcC52YXJpYW50cykge1xuICAgICAgICBlbnVtQ291bnQgPSBlbnVtQ291bnQgKyAxO1xuICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgYCAgJHtjb25zdGFudENhc2UodGhpcy5wcm90b2J1ZlR5cGVGb3JQcm9wKHByb3ApKX1fJHtjb25zdGFudENhc2UoXG4gICAgICAgICAgICB2YXJpYW50LFxuICAgICAgICAgICl9ID0gJHtlbnVtQ291bnR9O2AsXG4gICAgICAgICk7XG4gICAgICB9XG4gICAgICByZXN1bHRzLnB1c2goXCJ9XCIpO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9XG5cbiAgICBmb3IgKGNvbnN0IGJhZyBvZiBwcm9wLmJhZ05hbWVzKCkpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIGZvciAoY29uc3QgY2hpbGRQcm9wIG9mIHByb3BbYmFnXS5hdHRycykge1xuICAgICAgICBpZiAoY2hpbGRQcm9wIGluc3RhbmNlb2YgUHJvcE9iamVjdCB8fCBjaGlsZFByb3AgaW5zdGFuY2VvZiBQcm9wRW51bSkge1xuICAgICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLnByb3RvYnVmTWVzc2FnZUZvclByb3BPYmplY3QoY2hpbGRQcm9wKSk7XG4gICAgICAgIH1cbiAgICAgIH1cblxuICAgICAgY29uc3QgbWVzc2FnZU5hbWUgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLm5hbWUsXG4gICAgICApfWA7XG4gICAgICByZXN1bHRzLnB1c2goYG1lc3NhZ2UgJHttZXNzYWdlTmFtZX0ge2ApO1xuXG4gICAgICBsZXQgdW5pdmVyc2FsQmFzZSA9IDA7XG4gICAgICBsZXQgY3VzdG9tQmFzZSA9IDEwMDA7XG4gICAgICAvLyBAdHMtaWdub3JlXG4gICAgICBmb3IgKGNvbnN0IGluZGV4IGluIHByb3BbYmFnXS5hdHRycykge1xuICAgICAgICAvLyBAdHMtaWdub3JlXG4gICAgICAgIGNvbnN0IHAgPSBwcm9wW2JhZ10uYXR0cnNbaW5kZXhdO1xuXG4gICAgICAgIGlmIChwLnVuaXZlcnNhbCkge1xuICAgICAgICAgIHVuaXZlcnNhbEJhc2UgPSB1bml2ZXJzYWxCYXNlICsgMTtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXCIgIFwiICsgdGhpcy5wcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wKHAsIHVuaXZlcnNhbEJhc2UpKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBjdXN0b21CYXNlID0gY3VzdG9tQmFzZSArIDE7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKFwiICBcIiArIHRoaXMucHJvdG9idWZEZWZpbml0aW9uRm9yUHJvcChwLCBjdXN0b21CYXNlKSk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICAgIHJlc3VsdHMucHVzaChcIn1cIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBwcm90b2J1ZkltcG9ydHMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107IC8vIFRoaXMgY3JlYXRlcyBhIG5ld2xpbmUhXG4gICAgY29uc3QgcmVzdWx0U2V0ID0gdGhpcy5wcm90b2J1ZkltcG9ydFdhbGsoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdHMubWFwKHYgPT4gdi5yb290UHJvcCksXG4gICAgKTtcbiAgICBmb3IgKGNvbnN0IGxpbmUgb2YgcmVzdWx0U2V0LnZhbHVlcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goYGltcG9ydCBcIiR7bGluZX1cIjtgKTtcbiAgICB9XG4gICAgaWYgKHJlc3VsdHMubGVuZ3RoID4gMCkge1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiLy8gTm8gSW1wb3J0c1wiO1xuICAgIH1cbiAgfVxuXG4gIHByb3RvYnVmSW1wb3J0V2Fsayhwcm9wczogUHJvcHNbXSk6IFNldDxzdHJpbmc+IHtcbiAgICBjb25zdCByZXN1bHQ6IFNldDxzdHJpbmc+ID0gbmV3IFNldCgpO1xuXG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHByb3BzKSB7XG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgICAgY29uc3QgaW1wb3J0UGF0aCA9IHRoaXMucHJvdG9idWZJbXBvcnRGb3JQcm9wKHByb3ApO1xuICAgICAgICBpZiAoXG4gICAgICAgICAgaW1wb3J0UGF0aCAmJlxuICAgICAgICAgICFpbXBvcnRQYXRoLnN0YXJ0c1dpdGgoXG4gICAgICAgICAgICBgc2ktcmVnaXN0cnkvcHJvdG8vJHt0aGlzLnByb3RvYnVmUGFja2FnZU5hbWUoKX1gLFxuICAgICAgICAgIClcbiAgICAgICAgKSB7XG4gICAgICAgICAgcmVzdWx0LmFkZChpbXBvcnRQYXRoKTtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0LmFkZChcImdvb2dsZS9wcm90b2J1Zi93cmFwcGVycy5wcm90b1wiKTtcbiAgICAgIH1cblxuICAgICAgaWYgKHRoaXMucmVjdXJzZUtpbmRzLmluY2x1ZGVzKHByb3Aua2luZCgpKSkge1xuICAgICAgICBmb3IgKGNvbnN0IGJhZyBvZiBwcm9wLmJhZ05hbWVzKCkpIHtcbiAgICAgICAgICAvLyBAdHMtaWdub3JlXG4gICAgICAgICAgZm9yIChjb25zdCBpbnRlcm5hbFByb3Agb2YgcHJvcFtiYWddLmF0dHJzKSB7XG4gICAgICAgICAgICBjb25zdCBuZXdTZXQgPSB0aGlzLnByb3RvYnVmSW1wb3J0V2FsayhbaW50ZXJuYWxQcm9wXSk7XG4gICAgICAgICAgICBmb3IgKGNvbnN0IGl0ZW0gb2YgbmV3U2V0LnZhbHVlcygpKSB7XG4gICAgICAgICAgICAgIHJlc3VsdC5hZGQoaXRlbSk7XG4gICAgICAgICAgICB9XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQ7XG4gIH1cblxuICBnZW5lcmF0ZVN0cmluZygpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcHJvdG9idWYvcHJvdG8nLCB7IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IHRoaXMsXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gIH1cbn1cbiJdfQ==