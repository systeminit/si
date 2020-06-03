"use strict";

var _interopRequireWildcard = require("@babel/runtime/helpers/interopRequireWildcard");

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.ProtobufFormatter = void 0;

var _regenerator = _interopRequireDefault(require("@babel/runtime/regenerator"));

var _asyncToGenerator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncToGenerator"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _ejs = _interopRequireDefault(require("ejs"));

var _attrList = require("../attrList");

var _enum = require("../prop/enum");

var PropPrelude = _interopRequireWildcard(require("../components/prelude"));

var codeFs = _interopRequireWildcard(require("./fs"));

var _changeCase = require("change-case");

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

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
        return "map<string, google.protobuf.StringValue>"; // return "map<string, string>";
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
    key: "generateProto",
    value: function () {
      var _generateProto = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee() {
        return _regenerator["default"].wrap(function _callee$(_context) {
          while (1) {
            switch (_context.prev = _context.next) {
              case 0:
                _context.next = 2;
                return codeFs.writeCode("./proto/si.".concat(this.systemObjects[0].serviceName, ".proto"), this.generateString());

              case 2:
              case "end":
                return _context.stop();
            }
          }
        }, _callee, this);
      }));

      function generateProto() {
        return _generateProto.apply(this, arguments);
      }

      return generateProto;
    }()
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3Byb3RvYnVmLnRzIl0sIm5hbWVzIjpbIlByb3RvYnVmRm9ybWF0dGVyIiwic3lzdGVtT2JqZWN0cyIsImxlbmd0aCIsImZpcnN0Iiwic2VydmljZU5hbWUiLCJyZXN1bHRzIiwiZmlsdGVyIiwib2JqIiwibWV0aG9kc1Byb3AiLCJwcm9wZXJ0aWVzIiwicHVzaCIsIm9iamVjdCIsIm1ldGhvZHMiLCJhdHRycyIsIm1ldGhvZCIsIm1ldGhvZE5hbWUiLCJwYXJlbnROYW1lIiwibmFtZSIsImpvaW4iLCJwcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0Iiwicm9vdFByb3AiLCJtZXRob2RIb2xkZXIiLCJQcm9wUHJlbHVkZSIsIlByb3BNZXRob2QiLCJQcm9wQWN0aW9uIiwicmVxdWVzdCIsInJlcGx5IiwidHlwZU5hbWUiLCJwcm9wIiwiUHJvcExpbmsiLCJwcm9wT3duZXIiLCJsb29rdXBPYmplY3QiLCJwYXRoTmFtZSIsIlByb3BCb29sIiwiUHJvcENvZGUiLCJQcm9wRW51bSIsInJlYWxQcm9wIiwibG9va3VwTXlzZWxmIiwiUHJvcE9iamVjdCIsInByb3RvYnVmVHlwZUZvclByb3AiLCJQcm9wTWFwIiwiUHJvcE51bWJlciIsIm51bWJlcktpbmQiLCJwcm90b2J1ZlBhY2thZ2VOYW1lIiwiUHJvcFNlbGVjdCIsIlByb3BUZXh0Iiwia2luZCIsImlucHV0TnVtYmVyIiwicmVwZWF0ZWQiLCJlbnVtQ291bnQiLCJ2YXJpYW50cyIsInZhcmlhbnQiLCJiYWdOYW1lcyIsImJhZyIsImNoaWxkUHJvcCIsIm1lc3NhZ2VOYW1lIiwidW5pdmVyc2FsQmFzZSIsImN1c3RvbUJhc2UiLCJpbmRleCIsInAiLCJ1bml2ZXJzYWwiLCJwcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wIiwicmVzdWx0U2V0IiwicHJvdG9idWZJbXBvcnRXYWxrIiwibWFwIiwidiIsInZhbHVlcyIsImxpbmUiLCJwcm9wcyIsInJlc3VsdCIsIlNldCIsImltcG9ydFBhdGgiLCJwcm90b2J1ZkltcG9ydEZvclByb3AiLCJzdGFydHNXaXRoIiwiYWRkIiwicmVjdXJzZUtpbmRzIiwiaW5jbHVkZXMiLCJpbnRlcm5hbFByb3AiLCJuZXdTZXQiLCJpdGVtIiwiY29kZUZzIiwid3JpdGVDb2RlIiwiZ2VuZXJhdGVTdHJpbmciLCJlanMiLCJyZW5kZXIiLCJmbXQiLCJmaWxlbmFtZSJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBRUE7Ozs7Ozs7O0lBRWFBLGlCO0FBS1gsNkJBQVlDLGFBQVosRUFBMEM7QUFBQTtBQUFBO0FBQUEsMkRBRjNCLENBQUMsUUFBRCxDQUUyQjs7QUFDeEMsUUFBSUEsYUFBYSxDQUFDQyxNQUFkLElBQXdCLENBQTVCLEVBQStCO0FBQzdCLFlBQU0sc0NBQU47QUFDRDs7QUFDRCxTQUFLRCxhQUFMLEdBQXFCQSxhQUFyQjtBQUNEOzs7OzRCQUVvQjtBQUNuQixhQUFPLEtBQUtBLGFBQUwsQ0FBbUIsQ0FBbkIsQ0FBUDtBQUNEOzs7MENBRTZCO0FBQzVCLDBCQUFhLDJCQUFVLEtBQUtFLEtBQUwsR0FBYUMsV0FBdkIsQ0FBYjtBQUNEOzs7dUNBRTBCO0FBQ3pCLFVBQU1DLE9BQU8sR0FBRyxFQUFoQjs7QUFDQSxVQUNFLEtBQUtKLGFBQUwsQ0FBbUJLLE1BQW5CLENBQTBCLFVBQUFDLEdBQUc7QUFBQSxlQUFJQSxHQUFHLENBQUNDLFdBQUosQ0FBZ0JDLFVBQWhCLENBQTJCUCxNQUEzQixHQUFvQyxDQUF4QztBQUFBLE9BQTdCLEVBQ0dBLE1BREgsR0FDWSxDQUZkLEVBR0U7QUFDQUcsUUFBQUEsT0FBTyxDQUFDSyxJQUFSLG1CQUF3Qiw0QkFBVyxLQUFLUCxLQUFMLEdBQWFDLFdBQXhCLENBQXhCOztBQURBLG1EQUVxQixLQUFLSCxhQUYxQjtBQUFBOztBQUFBO0FBRUEsOERBQXlDO0FBQUEsZ0JBQTlCVSxNQUE4Qjs7QUFBQSx3REFDbEJBLE1BQU0sQ0FBQ0MsT0FB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aUI7QUFDdkIsYUFBT0MsZ0JBQUlDLE1BQUosQ0FDTCx1REFESyxFQUVMO0FBQ0VDLFFBQUFBLEdBQUcsRUFBRTtBQURQLE9BRkssRUFLTDtBQUNFQyxRQUFBQSxRQUFRLEVBQUU7QUFEWixPQUxLLENBQVA7QUFTRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IE9iamVjdFR5cGVzIH0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IGVqcyBmcm9tIFwiZWpzXCI7XG5pbXBvcnQgeyBQcm9wcywgUHJvcE9iamVjdCB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuaW1wb3J0IHsgUHJvcEVudW0gfSBmcm9tIFwiLi4vcHJvcC9lbnVtXCI7XG5pbXBvcnQgKiBhcyBQcm9wUHJlbHVkZSBmcm9tIFwiLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5pbXBvcnQgKiBhcyBjb2RlRnMgZnJvbSBcIi4vZnNcIjtcblxuaW1wb3J0IHsgc25ha2VDYXNlLCBwYXNjYWxDYXNlLCBjb25zdGFudENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcblxuZXhwb3J0IGNsYXNzIFByb3RvYnVmRm9ybWF0dGVyIHtcbiAgc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICByZWN1cnNlS2luZHMgPSBbXCJvYmplY3RcIl07XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXSkge1xuICAgIGlmIChzeXN0ZW1PYmplY3RzLmxlbmd0aCA9PSAwKSB7XG4gICAgICB0aHJvdyBcIllvdSBtdXN0IHByb3ZpZGUgb2JqZWN0cyB0byBnZW5lcmF0ZVwiO1xuICAgIH1cbiAgICB0aGlzLnN5c3RlbU9iamVjdHMgPSBzeXN0ZW1PYmplY3RzO1xuICB9XG5cbiAgZmlyc3QoKTogT2JqZWN0VHlwZXMge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHNbMF07XG4gIH1cblxuICBwcm90b2J1ZlBhY2thZ2VOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBzaS4ke3NuYWtlQ2FzZSh0aGlzLmZpcnN0KCkuc2VydmljZU5hbWUpfWA7XG4gIH1cblxuICBwcm90b2J1ZlNlcnZpY2VzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0cy5maWx0ZXIob2JqID0+IG9iai5tZXRob2RzUHJvcC5wcm9wZXJ0aWVzLmxlbmd0aCA+IDApXG4gICAgICAgIC5sZW5ndGggPiAwXG4gICAgKSB7XG4gICAgICByZXN1bHRzLnB1c2goYHNlcnZpY2UgJHtwYXNjYWxDYXNlKHRoaXMuZmlyc3QoKS5zZXJ2aWNlTmFtZSl9IHtgKTtcbiAgICAgIGZvciAoY29uc3Qgb2JqZWN0IG9mIHRoaXMuc3lzdGVtT2JqZWN0cykge1xuICAgICAgICBmb3IgKGNvbnN0IG1ldGhvZCBvZiBvYmplY3QubWV0aG9kcy5hdHRycykge1xuICAgICAgICAgIGNvbnN0IG1ldGhvZE5hbWUgPVxuICAgICAgICAgICAgcGFzY2FsQ2FzZShtZXRob2QucGFyZW50TmFtZSkgKyBwYXNjYWxDYXNlKG1ldGhvZC5uYW1lKTtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgICBgICBycGMgJHttZXRob2ROYW1lfSgke21ldGhvZE5hbWV9UmVxdWVzdCkgcmV0dXJucyAoJHttZXRob2ROYW1lfVJlcGx5KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICAgIHJlc3VsdHMucHVzaChgfWApO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9XG4gICAgcmV0dXJuIFwiLy8gTm8gU2VydmljZXNcIjtcbiAgfVxuXG4gIHByb3RvYnVmTWVzc2FnZXMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgZm9yIChjb25zdCBvYmplY3Qgb2YgdGhpcy5zeXN0ZW1PYmplY3RzKSB7XG4gICAgICByZXN1bHRzLnB1c2godGhpcy5wcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KG9iamVjdC5yb290UHJvcCkpO1xuICAgICAgaWYgKG9iamVjdC5tZXRob2RzUHJvcC5wcm9wZXJ0aWVzLmxlbmd0aCkge1xuICAgICAgICBmb3IgKGNvbnN0IG1ldGhvZEhvbGRlciBvZiBvYmplY3QubWV0aG9kc1Byb3AucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICAgIGlmIChcbiAgICAgICAgICAgIG1ldGhvZEhvbGRlciBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QgfHxcbiAgICAgICAgICAgIG1ldGhvZEhvbGRlciBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb25cbiAgICAgICAgICApIHtcbiAgICAgICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICAgICAgdGhpcy5wcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KG1ldGhvZEhvbGRlci5yZXF1ZXN0KSxcbiAgICAgICAgICAgICk7XG4gICAgICAgICAgICByZXN1bHRzLnB1c2godGhpcy5wcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KG1ldGhvZEhvbGRlci5yZXBseSkpO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICB0aHJvdyBgRXJyb3IgZ2VuZXJhdGluZyBwcm90b2J1ZiAtIG5vbiBtZXRob2QvYWN0aW9uIHByb3AgZm91bmQgb24gJHtvYmplY3QudHlwZU5hbWV9YDtcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHByb3RvYnVmSW1wb3J0Rm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgY29uc3QgcHJvcE93bmVyID0gcHJvcC5sb29rdXBPYmplY3QoKTtcbiAgICAgIGxldCBwYXRoTmFtZSA9IFwic2ktcmVnaXN0cnkvcHJvdG8vc2kuXCI7XG4gICAgICBpZiAocHJvcE93bmVyLnNlcnZpY2VOYW1lKSB7XG4gICAgICAgIHBhdGhOYW1lID0gcGF0aE5hbWUgKyBzbmFrZUNhc2UocHJvcE93bmVyLnNlcnZpY2VOYW1lKSArIFwiLnByb3RvXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICBwYXRoTmFtZSA9IHBhdGhOYW1lICsgc25ha2VDYXNlKHByb3BPd25lci50eXBlTmFtZSkgKyBcIi5wcm90b1wiO1xuICAgICAgfVxuICAgICAgcmV0dXJuIHBhdGhOYW1lO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJcIjtcbiAgICB9XG4gIH1cblxuICBwcm90b2J1ZlR5cGVGb3JQcm9wKHByb3A6IFByb3BzKTogc3RyaW5nIHtcbiAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BCb29sKSB7XG4gICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuQm9vbFZhbHVlXCI7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcENvZGUpIHtcbiAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5TdHJpbmdWYWx1ZVwiO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BFbnVtKSB7XG4gICAgICByZXR1cm4gYCR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgIGNvbnN0IHJlYWxQcm9wID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIGlmIChcbiAgICAgICAgcmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0IHx8XG4gICAgICAgIHJlYWxQcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEVudW1cbiAgICAgICkge1xuICAgICAgICBjb25zdCBwcm9wT3duZXIgPSBwcm9wLmxvb2t1cE9iamVjdCgpO1xuICAgICAgICBsZXQgcGF0aE5hbWUgPSBcInNpLlwiO1xuICAgICAgICBpZiAocHJvcE93bmVyLnNlcnZpY2VOYW1lKSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBwYXRoTmFtZSArIHNuYWtlQ2FzZShwcm9wT3duZXIuc2VydmljZU5hbWUpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHBhdGhOYW1lID0gcGF0aE5hbWUgKyBzbmFrZUNhc2UocHJvcE93bmVyLnR5cGVOYW1lKTtcbiAgICAgICAgfVxuICAgICAgICByZXR1cm4gYCR7cGF0aE5hbWV9LiR7cGFzY2FsQ2FzZShyZWFsUHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgICAgcmVhbFByb3AubmFtZSxcbiAgICAgICAgKX1gO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHRoaXMucHJvdG9idWZUeXBlRm9yUHJvcChyZWFsUHJvcCk7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1hcCkge1xuICAgICAgcmV0dXJuIFwibWFwPHN0cmluZywgZ29vZ2xlLnByb3RvYnVmLlN0cmluZ1ZhbHVlPlwiO1xuICAgICAgLy8gcmV0dXJuIFwibWFwPHN0cmluZywgc3RyaW5nPlwiO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BOdW1iZXIpIHtcbiAgICAgIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQzMlwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5JbnQzMlZhbHVlXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQzMlwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5VSW50MzJWYWx1ZVwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQ2NFwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5JbnQ2NFZhbHVlXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQ2NFwiKSB7XG4gICAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5VSW50NjRWYWx1ZVwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1MTI4XCIpIHtcbiAgICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLlN0cmluZ1ZhbHVlXCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgcmV0dXJuIGAke3RoaXMucHJvdG9idWZQYWNrYWdlTmFtZSgpfS4ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AucGFyZW50TmFtZSxcbiAgICAgICl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICByZXR1cm4gYCR7dGhpcy5wcm90b2J1ZlBhY2thZ2VOYW1lKCl9LiR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5wYXJlbnROYW1lLFxuICAgICAgKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24pIHtcbiAgICAgIHJldHVybiBgJHt0aGlzLnByb3RvYnVmUGFja2FnZU5hbWUoKX0uJHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLnBhcmVudE5hbWUsXG4gICAgICApfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wU2VsZWN0IHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFRleHRcbiAgICApIHtcbiAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5TdHJpbmdWYWx1ZVwiO1xuICAgIH0gZWxzZSB7XG4gICAgICAvLyBAdHMtaWdub3JlXG4gICAgICB0aHJvdyBgVW5rbm93biBwcm9wZXJ0eSB0eXBlIGZvciByZW5kZXJpbmcgcHJvdG9idWYhIEZpeCBtZTogJHtwcm9wLmtpbmQoKX1gO1xuICAgIH1cbiAgICByZXR1cm4gXCJ1bnJlYWNoYWJsZSFcIjtcbiAgfVxuXG4gIHByb3RvYnVmRGVmaW5pdGlvbkZvclByb3AocHJvcDogUHJvcHMsIGlucHV0TnVtYmVyOiBudW1iZXIpOiBzdHJpbmcge1xuICAgIGxldCByZXBlYXRlZDogc3RyaW5nO1xuICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICByZXBlYXRlZCA9IFwicmVwZWF0ZWQgXCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJlcGVhdGVkID0gXCJcIjtcbiAgICB9XG4gICAgcmV0dXJuIGAke3JlcGVhdGVkfSR7dGhpcy5wcm90b2J1ZlR5cGVGb3JQcm9wKHByb3ApfSAke3NuYWtlQ2FzZShcbiAgICAgIHByb3AubmFtZSxcbiAgICApfSA9ICR7aW5wdXROdW1iZXJ9O2A7XG4gIH1cblxuICBwcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KHByb3A6IFByb3BPYmplY3QgfCBQcm9wRW51bSk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuXG4gICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wRW51bSkge1xuICAgICAgbGV0IGVudW1Db3VudCA9IDA7XG4gICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgIGBlbnVtICR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfSB7YCxcbiAgICAgICk7XG4gICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgIGAgICR7Y29uc3RhbnRDYXNlKFxuICAgICAgICAgIHRoaXMucHJvdG9idWZUeXBlRm9yUHJvcChwcm9wKSxcbiAgICAgICAgKX1fVU5LTk9XTiA9ICR7ZW51bUNvdW50fTtgLFxuICAgICAgKTtcbiAgICAgIGZvciAoY29uc3QgdmFyaWFudCBvZiBwcm9wLnZhcmlhbnRzKSB7XG4gICAgICAgIGVudW1Db3VudCA9IGVudW1Db3VudCArIDE7XG4gICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICBgICAke2NvbnN0YW50Q2FzZSh0aGlzLnByb3RvYnVmVHlwZUZvclByb3AocHJvcCkpfV8ke2NvbnN0YW50Q2FzZShcbiAgICAgICAgICAgIHZhcmlhbnQsXG4gICAgICAgICAgKX0gPSAke2VudW1Db3VudH07YCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICAgIHJlc3VsdHMucHVzaChcIn1cIik7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH1cblxuICAgIGZvciAoY29uc3QgYmFnIG9mIHByb3AuYmFnTmFtZXMoKSkge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgZm9yIChjb25zdCBjaGlsZFByb3Agb2YgcHJvcFtiYWddLmF0dHJzKSB7XG4gICAgICAgIGlmIChjaGlsZFByb3AgaW5zdGFuY2VvZiBQcm9wT2JqZWN0IHx8IGNoaWxkUHJvcCBpbnN0YW5jZW9mIFByb3BFbnVtKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMucHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChjaGlsZFByb3ApKTtcbiAgICAgICAgfVxuICAgICAgfVxuXG4gICAgICBjb25zdCBtZXNzYWdlTmFtZSA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9YDtcbiAgICAgIHJlc3VsdHMucHVzaChgbWVzc2FnZSAke21lc3NhZ2VOYW1lfSB7YCk7XG5cbiAgICAgIGxldCB1bml2ZXJzYWxCYXNlID0gMDtcbiAgICAgIGxldCBjdXN0b21CYXNlID0gMTAwMDtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIGZvciAoY29uc3QgaW5kZXggaW4gcHJvcFtiYWddLmF0dHJzKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgY29uc3QgcCA9IHByb3BbYmFnXS5hdHRyc1tpbmRleF07XG5cbiAgICAgICAgaWYgKHAudW5pdmVyc2FsKSB7XG4gICAgICAgICAgdW5pdmVyc2FsQmFzZSA9IHVuaXZlcnNhbEJhc2UgKyAxO1xuICAgICAgICAgIHJlc3VsdHMucHVzaChcIiAgXCIgKyB0aGlzLnByb3RvYnVmRGVmaW5pdGlvbkZvclByb3AocCwgdW5pdmVyc2FsQmFzZSkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIGN1c3RvbUJhc2UgPSBjdXN0b21CYXNlICsgMTtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXCIgIFwiICsgdGhpcy5wcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wKHAsIGN1c3RvbUJhc2UpKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgICAgcmVzdWx0cy5wdXNoKFwifVwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHByb3RvYnVmSW1wb3J0cygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTsgLy8gVGhpcyBjcmVhdGVzIGEgbmV3bGluZSFcbiAgICBjb25zdCByZXN1bHRTZXQgPSB0aGlzLnByb3RvYnVmSW1wb3J0V2FsayhcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0cy5tYXAodiA9PiB2LnJvb3RQcm9wKSxcbiAgICApO1xuICAgIGZvciAoY29uc3QgbGluZSBvZiByZXN1bHRTZXQudmFsdWVzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChgaW1wb3J0IFwiJHtsaW5lfVwiO2ApO1xuICAgIH1cbiAgICBpZiAocmVzdWx0cy5sZW5ndGggPiAwKSB7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCIvLyBObyBJbXBvcnRzXCI7XG4gICAgfVxuICB9XG5cbiAgcHJvdG9idWZJbXBvcnRXYWxrKHByb3BzOiBQcm9wc1tdKTogU2V0PHN0cmluZz4ge1xuICAgIGNvbnN0IHJlc3VsdDogU2V0PHN0cmluZz4gPSBuZXcgU2V0KCk7XG5cbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcHMpIHtcbiAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcImxpbmtcIikge1xuICAgICAgICBjb25zdCBpbXBvcnRQYXRoID0gdGhpcy5wcm90b2J1ZkltcG9ydEZvclByb3AocHJvcCk7XG4gICAgICAgIGlmIChcbiAgICAgICAgICBpbXBvcnRQYXRoICYmXG4gICAgICAgICAgIWltcG9ydFBhdGguc3RhcnRzV2l0aChcbiAgICAgICAgICAgIGBzaS1yZWdpc3RyeS9wcm90by8ke3RoaXMucHJvdG9idWZQYWNrYWdlTmFtZSgpfWAsXG4gICAgICAgICAgKVxuICAgICAgICApIHtcbiAgICAgICAgICByZXN1bHQuYWRkKGltcG9ydFBhdGgpO1xuICAgICAgICB9XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHQuYWRkKFwiZ29vZ2xlL3Byb3RvYnVmL3dyYXBwZXJzLnByb3RvXCIpO1xuICAgICAgfVxuXG4gICAgICBpZiAodGhpcy5yZWN1cnNlS2luZHMuaW5jbHVkZXMocHJvcC5raW5kKCkpKSB7XG4gICAgICAgIGZvciAoY29uc3QgYmFnIG9mIHByb3AuYmFnTmFtZXMoKSkge1xuICAgICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgICBmb3IgKGNvbnN0IGludGVybmFsUHJvcCBvZiBwcm9wW2JhZ10uYXR0cnMpIHtcbiAgICAgICAgICAgIGNvbnN0IG5ld1NldCA9IHRoaXMucHJvdG9idWZJbXBvcnRXYWxrKFtpbnRlcm5hbFByb3BdKTtcbiAgICAgICAgICAgIGZvciAoY29uc3QgaXRlbSBvZiBuZXdTZXQudmFsdWVzKCkpIHtcbiAgICAgICAgICAgICAgcmVzdWx0LmFkZChpdGVtKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlUHJvdG8oKSB7XG4gICAgYXdhaXQgY29kZUZzLndyaXRlQ29kZShcbiAgICAgIGAuL3Byb3RvL3NpLiR7dGhpcy5zeXN0ZW1PYmplY3RzWzBdLnNlcnZpY2VOYW1lfS5wcm90b2AsXG4gICAgICB0aGlzLmdlbmVyYXRlU3RyaW5nKCksXG4gICAgKTtcbiAgfVxuXG4gIGdlbmVyYXRlU3RyaW5nKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9wcm90b2J1Zi9wcm90bycsIHsgZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogdGhpcyxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgfVxufVxuIl19