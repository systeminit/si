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
      throw new Error("You must provide objects to generate");
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
                  throw new Error("Error generating protobuf - non method/action prop found on ".concat(object.typeName));
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
        throw new Error("Unknown property type for rendering protobuf! Fix me");
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3Byb3RvYnVmLnRzIl0sIm5hbWVzIjpbIlByb3RvYnVmRm9ybWF0dGVyIiwic3lzdGVtT2JqZWN0cyIsImxlbmd0aCIsIkVycm9yIiwiZmlyc3QiLCJzZXJ2aWNlTmFtZSIsInJlc3VsdHMiLCJmaWx0ZXIiLCJvYmoiLCJtZXRob2RzUHJvcCIsInByb3BlcnRpZXMiLCJwdXNoIiwib2JqZWN0IiwibWV0aG9kcyIsImF0dHJzIiwibWV0aG9kIiwibWV0aG9kTmFtZSIsInBhcmVudE5hbWUiLCJuYW1lIiwiam9pbiIsInByb3RvYnVmTWVzc2FnZUZvclByb3BPYmplY3QiLCJyb290UHJvcCIsIm1ldGhvZEhvbGRlciIsIlByb3BQcmVsdWRlIiwiUHJvcE1ldGhvZCIsIlByb3BBY3Rpb24iLCJyZXF1ZXN0IiwicmVwbHkiLCJ0eXBlTmFtZSIsInByb3AiLCJQcm9wTGluayIsInByb3BPd25lciIsImxvb2t1cE9iamVjdCIsInBhdGhOYW1lIiwiUHJvcEJvb2wiLCJQcm9wQ29kZSIsIlByb3BFbnVtIiwicmVhbFByb3AiLCJsb29rdXBNeXNlbGYiLCJQcm9wT2JqZWN0IiwicHJvdG9idWZUeXBlRm9yUHJvcCIsIlByb3BNYXAiLCJQcm9wTnVtYmVyIiwibnVtYmVyS2luZCIsInByb3RvYnVmUGFja2FnZU5hbWUiLCJQcm9wU2VsZWN0IiwiUHJvcFRleHQiLCJpbnB1dE51bWJlciIsInJlcGVhdGVkIiwiZW51bUNvdW50IiwidmFyaWFudHMiLCJ2YXJpYW50IiwiYmFnTmFtZXMiLCJiYWciLCJjaGlsZFByb3AiLCJtZXNzYWdlTmFtZSIsInVuaXZlcnNhbEJhc2UiLCJjdXN0b21CYXNlIiwiaW5kZXgiLCJwIiwidW5pdmVyc2FsIiwicHJvdG9idWZEZWZpbml0aW9uRm9yUHJvcCIsInJlc3VsdFNldCIsInByb3RvYnVmSW1wb3J0V2FsayIsIm1hcCIsInYiLCJ2YWx1ZXMiLCJsaW5lIiwicHJvcHMiLCJyZXN1bHQiLCJTZXQiLCJraW5kIiwiaW1wb3J0UGF0aCIsInByb3RvYnVmSW1wb3J0Rm9yUHJvcCIsInN0YXJ0c1dpdGgiLCJhZGQiLCJyZWN1cnNlS2luZHMiLCJpbmNsdWRlcyIsImludGVybmFsUHJvcCIsIm5ld1NldCIsIml0ZW0iLCJjb2RlRnMiLCJ3cml0ZUNvZGUiLCJnZW5lcmF0ZVN0cmluZyIsImVqcyIsInJlbmRlciIsImZtdCIsImZpbGVuYW1lIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFFQTs7Ozs7Ozs7SUFFYUEsaUI7QUFLWCw2QkFBWUMsYUFBWixFQUEwQztBQUFBO0FBQUE7QUFBQSwyREFGM0IsQ0FBQyxRQUFELENBRTJCOztBQUN4QyxRQUFJQSxhQUFhLENBQUNDLE1BQWQsSUFBd0IsQ0FBNUIsRUFBK0I7QUFDN0IsWUFBTSxJQUFJQyxLQUFKLENBQVUsc0NBQVYsQ0FBTjtBQUNEOztBQUNELFNBQUtGLGFBQUwsR0FBcUJBLGFBQXJCO0FBQ0Q7Ozs7NEJBRW9CO0FBQ25CLGFBQU8sS0FBS0EsYUFBTCxDQUFtQixDQUFuQixDQUFQO0FBQ0Q7OzswQ0FFNkI7QUFDNUIsMEJBQWEsMkJBQVUsS0FBS0csS0FBTCxHQUFhQyxXQUF2QixDQUFiO0FBQ0Q7Ozt1Q0FFMEI7QUFDekIsVUFBTUMsT0FBTyxHQUFHLEVBQWhCOztBQUNBLFVBQ0UsS0FBS0wsYUFBTCxDQUFtQk0sTUFBbkIsQ0FBMEIsVUFBQUMsR0FBRztBQUFBLGVBQUlBLEdBQUcsQ0FBQ0MsV0FBSixDQUFnQkMsVUFBaEIsQ0FBMkJSLE1BQTNCLEdBQW9DLENBQXhDO0FBQUEsT0FBN0IsRUFDR0EsTUFESCxHQUNZLENBRmQsRUFHRTtBQUNBSSxRQUFBQSxPQUFPLENBQUNLLElBQVIsbUJBQXdCLDRCQUFXLEtBQUtQLEtBQUwsR0FBYUMsV0FBeEIsQ0FBeEI7O0FBREEsbURBRXFCLEtBQUtKLGFBRjFCO0FBQUE7O0FBQUE7QUFFQSw4REFBeUM7QUFBQSxnQkFBOUJXLE1BQThCOztBQUFBLHdEQUNsQkEsTUFBTSxDQUFDQyxPQUFQLENBQWVDLEtBREc7QUFBQTs7QUFBQTtBQUN2QyxxRUFBMkM7QUFBQSxvQkFBaENDLE1BQWdDO0FBQ3pDLG9CQUFNQyxVQUFVLEdBQ2QsNEJBQVdELE1BQU0sQ0FBQ0UsVUFBbEIsSUFBZ0MsNEJBQVdGLE1BQU0sQ0FBQ0csSUFBbEIsQ0FEbEM7QUFFQVosZ0JBQUFBLE9BQU8sQ0FBQ0ssSUFBUixpQkFDV0ssVUFEWCxjQUN5QkEsVUFEekIsK0JBQ3dEQSxVQUR4RDtBQUdEO0FBUHNDO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFReEM7QUFWRDtBQUFBO0FBQUE7QUFBQTtBQUFBOztBQVdBVixRQUFBQSxPQUFPLENBQUNLLElBQVI7QUFDQSxlQUFPTCxPQUFPLENBQUNhLElBQVIsQ0FBYSxJQUFiLENBQVA7QUFDRDs7QUFDRCxhQUFPLGdCQUFQO0FBQ0Q7Ozt1Q0FFMEI7QUFDekIsVUFBTWIsT0FBTyxHQUFHLEVBQWhCOztBQUR5QixrREFFSixLQUFLTCxhQUZEO0FBQUE7O0FBQUE7QUFFekIsK0RBQXlDO0FBQUEsY0FBOUJXLE1BQThCO0FBQ3ZDTixVQUFBQSxPQUFPLENBQUNLLElBQVIsQ0FBYSxLQUFLUyw0QkFBTCxDQUFrQ1IsTUFBTSxDQUFDUyxRQUF6QyxDQUFiOztBQUNBLGNBQUlULE1BQU0sQ0FBQ0gsV0FBUCxDQUFtQkMsVUFBbkIsQ0FBOEJSLE1BQWxDLEVBQTBDO0FBQUEsd0RBQ2JVLE1BQU0sQ0FBQ0gsV0FBUCxDQUFtQkMsVUFBbkIsQ0FBOEJJLEtBRGpCO0FBQUE7O0FBQUE7QUFDeEMscUVBQWdFO0FBQUEsb0JBQXJEUSxZQUFxRDs7QUFDOUQsb0JBQ0VBLFlBQVksWUFBWUMsV0FBVyxDQUFDQyxVQUFwQyxJQUNBRixZQUFZLFlBQVlDLFdBQVcsQ0FBQ0UsVUFGdEMsRUFHRTtBQUNBbkIsa0JBQUFBLE9BQU8sQ0FBQ0ssSUFBUixDQUNFLEtBQUtTLDRCQUFMLENBQWtDRSxZQUFZLENBQUNJLE9BQS9DLENBREY7QUFHQXBCLGtCQUFBQSxPQUFPLENBQUNLLElBQVIsQ0FBYSxLQUFLUyw0QkFBTCxDQUFrQ0UsWUFBWSxDQUFDSyxLQUEvQyxDQUFiO0FBQ0QsaUJBUkQsTUFRTztBQUNMLHdCQUFNLElBQUl4QixLQUFKLHVFQUMyRFMsTUFBTSxDQUFDZ0IsUUFEbEUsRUFBTjtBQUdEO0FBQ0Y7QUFmdUM7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQWdCekM7QUFDRjtBQXJCd0I7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFzQnpCLGFBQU90QixPQUFPLENBQUNhLElBQVIsQ0FBYSxJQUFiLENBQVA7QUFDRDs7OzBDQUVxQlUsSSxFQUFxQjtBQUN6QyxVQUFJQSxJQUFJLFlBQVlOLFdBQVcsQ0FBQ08sUUFBaEMsRUFBMEM7QUFDeEMsWUFBTUMsU0FBUyxHQUFHRixJQUFJLENBQUNHLFlBQUwsRUFBbEI7QUFDQSxZQUFJQyxRQUFRLEdBQUcsdUJBQWY7O0FBQ0EsWUFBSUYsU0FBUyxDQUFDMUIsV0FBZCxFQUEyQjtBQUN6QjRCLFVBQUFBLFFBQVEsR0FBR0EsUUFBUSxHQUFHLDJCQUFVRixTQUFTLENBQUMxQixXQUFwQixDQUFYLEdBQThDLFFBQXpEO0FBQ0QsU0FGRCxNQUVPO0FBQ0w0QixVQUFBQSxRQUFRLEdBQUdBLFFBQVEsR0FBRywyQkFBVUYsU0FBUyxDQUFDSCxRQUFwQixDQUFYLEdBQTJDLFFBQXREO0FBQ0Q7O0FBQ0QsZUFBT0ssUUFBUDtBQUNELE9BVEQsTUFTTztBQUNMLGVBQU8sRUFBUDtBQUNEO0FBQ0Y7Ozt3Q0FFbUJKLEksRUFBcUI7QUFDdkMsVUFBSUEsSUFBSSxZQUFZTixXQUFXLENBQUNXLFFBQWhDLEVBQTBDO0FBQ3hDLGVBQU8sMkJBQVA7QUFDRCxPQUZELE1BRU8sSUFBSUwsSUFBSSxZQUFZTixXQUFXLENBQUNZLFFBQWhDLEVBQTBDO0FBQy9DLGVBQU8sNkJBQVA7QUFDRCxPQUZNLE1BRUEsSUFBSU4sSUFBSSxZQUFZTixXQUFXLENBQUNhLFFBQWhDLEVBQTBDO0FBQy9DLHlCQUFVLDRCQUFXUCxJQUFJLENBQUNaLFVBQWhCLENBQVYsU0FBd0MsNEJBQVdZLElBQUksQ0FBQ1gsSUFBaEIsQ0FBeEM7QUFDRCxPQUZNLE1BRUEsSUFBSVcsSUFBSSxZQUFZTixXQUFXLENBQUNPLFFBQWhDLEVBQTBDO0FBQy9DLFlBQU1PLFFBQVEsR0FBR1IsSUFBSSxDQUFDUyxZQUFMLEVBQWpCOztBQUNBLFlBQ0VELFFBQVEsWUFBWWQsV0FBVyxDQUFDZ0IsVUFBaEMsSUFDQUYsUUFBUSxZQUFZZCxXQUFXLENBQUNhLFFBRmxDLEVBR0U7QUFDQSxjQUFNTCxTQUFTLEdBQUdGLElBQUksQ0FBQ0csWUFBTCxFQUFsQjtBQUNBLGNBQUlDLFFBQVEsR0FBRyxLQUFmOztBQUNBLGNBQUlGLFNBQVMsQ0FBQzFCLFdBQWQsRUFBMkI7QUFDekI0QixZQUFBQSxRQUFRLEdBQUdBLFFBQVEsR0FBRywyQkFBVUYsU0FBUyxDQUFDMUIsV0FBcEIsQ0FBdEI7QUFDRCxXQUZELE1BRU87QUFDTDRCLFlBQUFBLFFBQVEsR0FBR0EsUUFBUSxHQUFHLDJCQUFVRixTQUFTLENBQUNILFFBQXBCLENBQXRCO0FBQ0Q7O0FBQ0QsMkJBQVVLLFFBQVYsY0FBc0IsNEJBQVdJLFFBQVEsQ0FBQ3BCLFVBQXBCLENBQXRCLFNBQXdELDRCQUN0RG9CLFFBQVEsQ0FBQ25CLElBRDZDLENBQXhEO0FBR0QsU0FkRCxNQWNPO0FBQ0wsaUJBQU8sS0FBS3NCLG1CQUFMLENBQXlCSCxRQUF6QixDQUFQO0FBQ0Q7QUFDRixPQW5CTSxNQW1CQSxJQUFJUixJQUFJLFlBQVlOLFdBQVcsQ0FBQ2tCLE9BQWhDLEVBQXlDO0FBQzlDLGVBQU8sMENBQVAsQ0FEOEMsQ0FFOUM7QUFDRCxPQUhNLE1BR0EsSUFBSVosSUFBSSxZQUFZTixXQUFXLENBQUNtQixVQUFoQyxFQUE0QztBQUNqRCxZQUFJYixJQUFJLENBQUNjLFVBQUwsSUFBbUIsT0FBdkIsRUFBZ0M7QUFDOUIsaUJBQU8sNEJBQVA7QUFDRCxTQUZELE1BRU8sSUFBSWQsSUFBSSxDQUFDYyxVQUFMLElBQW1CLFFBQXZCLEVBQWlDO0FBQ3RDLGlCQUFPLDZCQUFQO0FBQ0QsU0FGTSxNQUVBLElBQUlkLElBQUksQ0FBQ2MsVUFBTCxJQUFtQixPQUF2QixFQUFnQztBQUNyQyxpQkFBTyw0QkFBUDtBQUNELFNBRk0sTUFFQSxJQUFJZCxJQUFJLENBQUNjLFVBQUwsSUFBbUIsUUFBdkIsRUFBaUM7QUFDdEMsaUJBQU8sNkJBQVA7QUFDRCxTQUZNLE1BRUEsSUFBSWQsSUFBSSxDQUFDYyxVQUFMLElBQW1CLE1BQXZCLEVBQStCO0FBQ3BDLGlCQUFPLDZCQUFQO0FBQ0Q7QUFDRixPQVpNLE1BWUEsSUFBSWQsSUFBSSxZQUFZTixXQUFXLENBQUNnQixVQUFoQyxFQUE0QztBQUNqRCx5QkFBVSxLQUFLSyxtQkFBTCxFQUFWLGNBQXdDLDRCQUN0Q2YsSUFBSSxDQUFDWixVQURpQyxDQUF4QyxTQUVJLDRCQUFXWSxJQUFJLENBQUNYLElBQWhCLENBRko7QUFHRCxPQUpNLE1BSUEsSUFBSVcsSUFBSSxZQUFZTixXQUFXLENBQUNDLFVBQWhDLEVBQTRDO0FBQ2pELHlCQUFVLEtBQUtvQixtQkFBTCxFQUFWLGNBQXdDLDRCQUN0Q2YsSUFBSSxDQUFDWixVQURpQyxDQUF4QyxTQUVJLDRCQUFXWSxJQUFJLENBQUNYLElBQWhCLENBRko7QUFHRCxPQUpNLE1BSUEsSUFBSVcsSUFBSSxZQUFZTixXQUFXLENBQUNFLFVBQWhDLEVBQTRDO0FBQ2pELHlCQUFVLEtBQUttQixtQkFBTCxFQUFWLGNBQXdDLDRCQUN0Q2YsSUFBSSxDQUFDWixVQURpQyxDQUF4QyxTQUVJLDRCQUFXWSxJQUFJLENBQUNYLElBQWhCLENBRko7QUFHRCxPQUpNLE1BSUEsSUFDTFcsSUFBSSxZQUFZTixXQUFXLENBQUNzQixVQUE1QixJQUNBaEIsSUFBSSxZQUFZTixXQUFXLENBQUN1QixRQUZ2QixFQUdMO0FBQ0EsZUFBTyw2QkFBUDtBQUNELE9BTE0sTUFLQTtBQUNMO0FBQ0EsY0FBTSxJQUFJM0MsS0FBSix3REFBTjtBQUNEOztBQUNELGFBQU8sY0FBUDtBQUNEOzs7OENBRXlCMEIsSSxFQUFha0IsVyxFQUE2QjtBQUNsRSxVQUFJQyxRQUFKOztBQUNBLFVBQUluQixJQUFJLENBQUNtQixRQUFULEVBQW1CO0FBQ2pCQSxRQUFBQSxRQUFRLEdBQUcsV0FBWDtBQUNELE9BRkQsTUFFTztBQUNMQSxRQUFBQSxRQUFRLEdBQUcsRUFBWDtBQUNEOztBQUNELHVCQUFVQSxRQUFWLFNBQXFCLEtBQUtSLG1CQUFMLENBQXlCWCxJQUF6QixDQUFyQixjQUF1RCwyQkFDckRBLElBQUksQ0FBQ1gsSUFEZ0QsQ0FBdkQsZ0JBRU82QixXQUZQO0FBR0Q7OztpREFFNEJsQixJLEVBQXFDO0FBQ2hFLFVBQU12QixPQUFPLEdBQUcsRUFBaEI7O0FBRUEsVUFBSXVCLElBQUksWUFBWU8sY0FBcEIsRUFBOEI7QUFDNUIsWUFBSWEsU0FBUyxHQUFHLENBQWhCO0FBQ0EzQyxRQUFBQSxPQUFPLENBQUNLLElBQVIsZ0JBQ1UsNEJBQVdrQixJQUFJLENBQUNaLFVBQWhCLENBRFYsU0FDd0MsNEJBQVdZLElBQUksQ0FBQ1gsSUFBaEIsQ0FEeEM7QUFHQVosUUFBQUEsT0FBTyxDQUFDSyxJQUFSLGFBQ08sOEJBQ0gsS0FBSzZCLG1CQUFMLENBQXlCWCxJQUF6QixDQURHLENBRFAsd0JBR2lCb0IsU0FIakI7O0FBTDRCLG9EQVVOcEIsSUFBSSxDQUFDcUIsUUFWQztBQUFBOztBQUFBO0FBVTVCLGlFQUFxQztBQUFBLGdCQUExQkMsT0FBMEI7QUFDbkNGLFlBQUFBLFNBQVMsR0FBR0EsU0FBUyxHQUFHLENBQXhCO0FBQ0EzQyxZQUFBQSxPQUFPLENBQUNLLElBQVIsYUFDTyw4QkFBYSxLQUFLNkIsbUJBQUwsQ0FBeUJYLElBQXpCLENBQWIsQ0FEUCxjQUN1RCw4QkFDbkRzQixPQURtRCxDQUR2RCxnQkFHU0YsU0FIVDtBQUtEO0FBakIyQjtBQUFBO0FBQUE7QUFBQTtBQUFBOztBQWtCNUIzQyxRQUFBQSxPQUFPLENBQUNLLElBQVIsQ0FBYSxHQUFiO0FBQ0EsZUFBT0wsT0FBTyxDQUFDYSxJQUFSLENBQWEsSUFBYixDQUFQO0FBQ0Q7O0FBdkIrRCxrREF5QjlDVSxJQUFJLENBQUN1QixRQUFMLEVBekI4QztBQUFBOztBQUFBO0FBeUJoRSwrREFBbUM7QUFBQSxjQUF4QkMsR0FBd0I7O0FBQ2pDO0FBRGlDLHNEQUVUeEIsSUFBSSxDQUFDd0IsR0FBRCxDQUFKLENBQVV2QyxLQUZEO0FBQUE7O0FBQUE7QUFFakMsbUVBQXlDO0FBQUEsa0JBQTlCd0MsU0FBOEI7O0FBQ3ZDLGtCQUFJQSxTQUFTLFlBQVlmLG9CQUFyQixJQUFtQ2UsU0FBUyxZQUFZbEIsY0FBNUQsRUFBc0U7QUFDcEU5QixnQkFBQUEsT0FBTyxDQUFDSyxJQUFSLENBQWEsS0FBS1MsNEJBQUwsQ0FBa0NrQyxTQUFsQyxDQUFiO0FBQ0Q7QUFDRjtBQU5nQztBQUFBO0FBQUE7QUFBQTtBQUFBOztBQVFqQyxjQUFNQyxXQUFXLGFBQU0sNEJBQVcxQixJQUFJLENBQUNaLFVBQWhCLENBQU4sU0FBb0MsNEJBQ25EWSxJQUFJLENBQUNYLElBRDhDLENBQXBDLENBQWpCO0FBR0FaLFVBQUFBLE9BQU8sQ0FBQ0ssSUFBUixtQkFBd0I0QyxXQUF4QjtBQUVBLGNBQUlDLGFBQWEsR0FBRyxDQUFwQjtBQUNBLGNBQUlDLFVBQVUsR0FBRyxJQUFqQixDQWRpQyxDQWVqQzs7QUFDQSxlQUFLLElBQU1DLEtBQVgsSUFBb0I3QixJQUFJLENBQUN3QixHQUFELENBQUosQ0FBVXZDLEtBQTlCLEVBQXFDO0FBQ25DO0FBQ0EsZ0JBQU02QyxDQUFDLEdBQUc5QixJQUFJLENBQUN3QixHQUFELENBQUosQ0FBVXZDLEtBQVYsQ0FBZ0I0QyxLQUFoQixDQUFWOztBQUVBLGdCQUFJQyxDQUFDLENBQUNDLFNBQU4sRUFBaUI7QUFDZkosY0FBQUEsYUFBYSxHQUFHQSxhQUFhLEdBQUcsQ0FBaEM7QUFDQWxELGNBQUFBLE9BQU8sQ0FBQ0ssSUFBUixDQUFhLE9BQU8sS0FBS2tELHlCQUFMLENBQStCRixDQUEvQixFQUFrQ0gsYUFBbEMsQ0FBcEI7QUFDRCxhQUhELE1BR087QUFDTEMsY0FBQUEsVUFBVSxHQUFHQSxVQUFVLEdBQUcsQ0FBMUI7QUFDQW5ELGNBQUFBLE9BQU8sQ0FBQ0ssSUFBUixDQUFhLE9BQU8sS0FBS2tELHlCQUFMLENBQStCRixDQUEvQixFQUFrQ0YsVUFBbEMsQ0FBcEI7QUFDRDtBQUNGOztBQUNEbkQsVUFBQUEsT0FBTyxDQUFDSyxJQUFSLENBQWEsR0FBYjtBQUNEO0FBdEQrRDtBQUFBO0FBQUE7QUFBQTtBQUFBOztBQXVEaEUsYUFBT0wsT0FBTyxDQUFDYSxJQUFSLENBQWEsSUFBYixDQUFQO0FBQ0Q7OztzQ0FFeUI7QUFDeEIsVUFBTWIsT0FBTyxHQUFHLEVBQWhCLENBRHdCLENBQ0o7O0FBQ3BCLFVBQU13RCxTQUFTLEdBQUcsS0FBS0Msa0JBQUwsQ0FDaEIsS0FBSzlELGFBQUwsQ0FBbUIrRCxHQUFuQixDQUF1QixVQUFBQyxDQUFDO0FBQUEsZUFBSUEsQ0FBQyxDQUFDNUMsUUFBTjtBQUFBLE9BQXhCLENBRGdCLENBQWxCOztBQUZ3QixrREFLTHlDLFNBQVMsQ0FBQ0ksTUFBVixFQUxLO0FBQUE7O0FBQUE7QUFLeEIsK0RBQXVDO0FBQUEsY0FBNUJDLElBQTRCO0FBQ3JDN0QsVUFBQUEsT0FBTyxDQUFDSyxJQUFSLG9CQUF3QndELElBQXhCO0FBQ0Q7QUFQdUI7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFReEIsVUFBSTdELE9BQU8sQ0FBQ0osTUFBUixHQUFpQixDQUFyQixFQUF3QjtBQUN0QixlQUFPSSxPQUFPLENBQUNhLElBQVIsQ0FBYSxJQUFiLENBQVA7QUFDRCxPQUZELE1BRU87QUFDTCxlQUFPLGVBQVA7QUFDRDtBQUNGOzs7dUNBRWtCaUQsSyxFQUE2QjtBQUM5QyxVQUFNQyxNQUFtQixHQUFHLElBQUlDLEdBQUosRUFBNUI7O0FBRDhDLGtEQUczQkYsS0FIMkI7QUFBQTs7QUFBQTtBQUc5QywrREFBMEI7QUFBQSxjQUFmdkMsSUFBZTs7QUFDeEIsY0FBSUEsSUFBSSxDQUFDMEMsSUFBTCxNQUFlLE1BQW5CLEVBQTJCO0FBQ3pCLGdCQUFNQyxVQUFVLEdBQUcsS0FBS0MscUJBQUwsQ0FBMkI1QyxJQUEzQixDQUFuQjs7QUFDQSxnQkFDRTJDLFVBQVUsSUFDVixDQUFDQSxVQUFVLENBQUNFLFVBQVgsNkJBQ3NCLEtBQUs5QixtQkFBTCxFQUR0QixFQUZILEVBS0U7QUFDQXlCLGNBQUFBLE1BQU0sQ0FBQ00sR0FBUCxDQUFXSCxVQUFYO0FBQ0Q7QUFDRixXQVZELE1BVU87QUFDTEgsWUFBQUEsTUFBTSxDQUFDTSxHQUFQLENBQVcsZ0NBQVg7QUFDRDs7QUFFRCxjQUFJLEtBQUtDLFlBQUwsQ0FBa0JDLFFBQWxCLENBQTJCaEQsSUFBSSxDQUFDMEMsSUFBTCxFQUEzQixDQUFKLEVBQTZDO0FBQUEseURBQ3pCMUMsSUFBSSxDQUFDdUIsUUFBTCxFQUR5QjtBQUFBOztBQUFBO0FBQzNDLHdFQUFtQztBQUFBLG9CQUF4QkMsR0FBd0I7O0FBQ2pDO0FBRGlDLDZEQUVOeEIsSUFBSSxDQUFDd0IsR0FBRCxDQUFKLENBQVV2QyxLQUZKO0FBQUE7O0FBQUE7QUFFakMsNEVBQTRDO0FBQUEsd0JBQWpDZ0UsWUFBaUM7QUFDMUMsd0JBQU1DLE1BQU0sR0FBRyxLQUFLaEIsa0JBQUwsQ0FBd0IsQ0FBQ2UsWUFBRCxDQUF4QixDQUFmOztBQUQwQyxpRUFFdkJDLE1BQU0sQ0FBQ2IsTUFBUCxFQUZ1QjtBQUFBOztBQUFBO0FBRTFDLGdGQUFvQztBQUFBLDRCQUF6QmMsSUFBeUI7QUFDbENYLHdCQUFBQSxNQUFNLENBQUNNLEdBQVAsQ0FBV0ssSUFBWDtBQUNEO0FBSnlDO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFLM0M7QUFQZ0M7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQVFsQztBQVQwQztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBVTVDO0FBQ0Y7QUE3QjZDO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBOEI5QyxhQUFPWCxNQUFQO0FBQ0Q7Ozs7Ozs7Ozs7dUJBR09ZLE1BQU0sQ0FBQ0MsU0FBUCxzQkFDVSxLQUFLakYsYUFBTCxDQUFtQixDQUFuQixFQUFzQkksV0FEaEMsYUFFSixLQUFLOEUsY0FBTCxFQUZJLEM7Ozs7Ozs7Ozs7Ozs7Ozs7OztxQ0FNaUI7QUFDdkIsYUFBT0MsZ0JBQUlDLE1BQUosQ0FDTCx1REFESyxFQUVMO0FBQ0VDLFFBQUFBLEdBQUcsRUFBRTtBQURQLE9BRkssRUFLTDtBQUNFQyxRQUFBQSxRQUFRLEVBQUU7QUFEWixPQUxLLENBQVA7QUFTRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IE9iamVjdFR5cGVzIH0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IGVqcyBmcm9tIFwiZWpzXCI7XG5pbXBvcnQgeyBQcm9wcywgUHJvcE9iamVjdCB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuaW1wb3J0IHsgUHJvcEVudW0gfSBmcm9tIFwiLi4vcHJvcC9lbnVtXCI7XG5pbXBvcnQgKiBhcyBQcm9wUHJlbHVkZSBmcm9tIFwiLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5pbXBvcnQgKiBhcyBjb2RlRnMgZnJvbSBcIi4vZnNcIjtcblxuaW1wb3J0IHsgc25ha2VDYXNlLCBwYXNjYWxDYXNlLCBjb25zdGFudENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcblxuZXhwb3J0IGNsYXNzIFByb3RvYnVmRm9ybWF0dGVyIHtcbiAgc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICByZWN1cnNlS2luZHMgPSBbXCJvYmplY3RcIl07XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXSkge1xuICAgIGlmIChzeXN0ZW1PYmplY3RzLmxlbmd0aCA9PSAwKSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoXCJZb3UgbXVzdCBwcm92aWRlIG9iamVjdHMgdG8gZ2VuZXJhdGVcIik7XG4gICAgfVxuICAgIHRoaXMuc3lzdGVtT2JqZWN0cyA9IHN5c3RlbU9iamVjdHM7XG4gIH1cblxuICBmaXJzdCgpOiBPYmplY3RUeXBlcyB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0c1swXTtcbiAgfVxuXG4gIHByb3RvYnVmUGFja2FnZU5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYHNpLiR7c25ha2VDYXNlKHRoaXMuZmlyc3QoKS5zZXJ2aWNlTmFtZSl9YDtcbiAgfVxuXG4gIHByb3RvYnVmU2VydmljZXMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3RzLmZpbHRlcihvYmogPT4gb2JqLm1ldGhvZHNQcm9wLnByb3BlcnRpZXMubGVuZ3RoID4gMClcbiAgICAgICAgLmxlbmd0aCA+IDBcbiAgICApIHtcbiAgICAgIHJlc3VsdHMucHVzaChgc2VydmljZSAke3Bhc2NhbENhc2UodGhpcy5maXJzdCgpLnNlcnZpY2VOYW1lKX0ge2ApO1xuICAgICAgZm9yIChjb25zdCBvYmplY3Qgb2YgdGhpcy5zeXN0ZW1PYmplY3RzKSB7XG4gICAgICAgIGZvciAoY29uc3QgbWV0aG9kIG9mIG9iamVjdC5tZXRob2RzLmF0dHJzKSB7XG4gICAgICAgICAgY29uc3QgbWV0aG9kTmFtZSA9XG4gICAgICAgICAgICBwYXNjYWxDYXNlKG1ldGhvZC5wYXJlbnROYW1lKSArIHBhc2NhbENhc2UobWV0aG9kLm5hbWUpO1xuICAgICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICAgIGAgIHJwYyAke21ldGhvZE5hbWV9KCR7bWV0aG9kTmFtZX1SZXF1ZXN0KSByZXR1cm5zICgke21ldGhvZE5hbWV9UmVwbHkpO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgICAgcmVzdWx0cy5wdXNoKGB9YCk7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH1cbiAgICByZXR1cm4gXCIvLyBObyBTZXJ2aWNlc1wiO1xuICB9XG5cbiAgcHJvdG9idWZNZXNzYWdlcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICBmb3IgKGNvbnN0IG9iamVjdCBvZiB0aGlzLnN5c3RlbU9iamVjdHMpIHtcbiAgICAgIHJlc3VsdHMucHVzaCh0aGlzLnByb3RvYnVmTWVzc2FnZUZvclByb3BPYmplY3Qob2JqZWN0LnJvb3RQcm9wKSk7XG4gICAgICBpZiAob2JqZWN0Lm1ldGhvZHNQcm9wLnByb3BlcnRpZXMubGVuZ3RoKSB7XG4gICAgICAgIGZvciAoY29uc3QgbWV0aG9kSG9sZGVyIG9mIG9iamVjdC5tZXRob2RzUHJvcC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgICAgaWYgKFxuICAgICAgICAgICAgbWV0aG9kSG9sZGVyIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCB8fFxuICAgICAgICAgICAgbWV0aG9kSG9sZGVyIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvblxuICAgICAgICAgICkge1xuICAgICAgICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICAgICAgICB0aGlzLnByb3RvYnVmTWVzc2FnZUZvclByb3BPYmplY3QobWV0aG9kSG9sZGVyLnJlcXVlc3QpLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLnByb3RvYnVmTWVzc2FnZUZvclByb3BPYmplY3QobWV0aG9kSG9sZGVyLnJlcGx5KSk7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgICAgICAgYEVycm9yIGdlbmVyYXRpbmcgcHJvdG9idWYgLSBub24gbWV0aG9kL2FjdGlvbiBwcm9wIGZvdW5kIG9uICR7b2JqZWN0LnR5cGVOYW1lfWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgcHJvdG9idWZJbXBvcnRGb3JQcm9wKHByb3A6IFByb3BzKTogc3RyaW5nIHtcbiAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCBwcm9wT3duZXIgPSBwcm9wLmxvb2t1cE9iamVjdCgpO1xuICAgICAgbGV0IHBhdGhOYW1lID0gXCJzaS1yZWdpc3RyeS9wcm90by9zaS5cIjtcbiAgICAgIGlmIChwcm9wT3duZXIuc2VydmljZU5hbWUpIHtcbiAgICAgICAgcGF0aE5hbWUgPSBwYXRoTmFtZSArIHNuYWtlQ2FzZShwcm9wT3duZXIuc2VydmljZU5hbWUpICsgXCIucHJvdG9cIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHBhdGhOYW1lID0gcGF0aE5hbWUgKyBzbmFrZUNhc2UocHJvcE93bmVyLnR5cGVOYW1lKSArIFwiLnByb3RvXCI7XG4gICAgICB9XG4gICAgICByZXR1cm4gcGF0aE5hbWU7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlwiO1xuICAgIH1cbiAgfVxuXG4gIHByb3RvYnVmVHlwZUZvclByb3AocHJvcDogUHJvcHMpOiBzdHJpbmcge1xuICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEJvb2wpIHtcbiAgICAgIHJldHVybiBcImdvb2dsZS5wcm90b2J1Zi5Cb29sVmFsdWVcIjtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSkge1xuICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLlN0cmluZ1ZhbHVlXCI7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEVudW0pIHtcbiAgICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgY29uc3QgcmVhbFByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgaWYgKFxuICAgICAgICByZWFsUHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QgfHxcbiAgICAgICAgcmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wRW51bVxuICAgICAgKSB7XG4gICAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICAgIGxldCBwYXRoTmFtZSA9IFwic2kuXCI7XG4gICAgICAgIGlmIChwcm9wT3duZXIuc2VydmljZU5hbWUpIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IHBhdGhOYW1lICsgc25ha2VDYXNlKHByb3BPd25lci5zZXJ2aWNlTmFtZSk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBwYXRoTmFtZSArIHNuYWtlQ2FzZShwcm9wT3duZXIudHlwZU5hbWUpO1xuICAgICAgICB9XG4gICAgICAgIHJldHVybiBgJHtwYXRoTmFtZX0uJHtwYXNjYWxDYXNlKHJlYWxQcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgICByZWFsUHJvcC5uYW1lLFxuICAgICAgICApfWA7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXR1cm4gdGhpcy5wcm90b2J1ZlR5cGVGb3JQcm9wKHJlYWxQcm9wKTtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWFwKSB7XG4gICAgICByZXR1cm4gXCJtYXA8c3RyaW5nLCBnb29nbGUucHJvdG9idWYuU3RyaW5nVmFsdWU+XCI7XG4gICAgICAvLyByZXR1cm4gXCJtYXA8c3RyaW5nLCBzdHJpbmc+XCI7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE51bWJlcikge1xuICAgICAgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcImludDMyXCIpIHtcbiAgICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLkludDMyVmFsdWVcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDMyXCIpIHtcbiAgICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLlVJbnQzMlZhbHVlXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcImludDY0XCIpIHtcbiAgICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLkludDY0VmFsdWVcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDY0XCIpIHtcbiAgICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLlVJbnQ2NFZhbHVlXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInUxMjhcIikge1xuICAgICAgICByZXR1cm4gXCJnb29nbGUucHJvdG9idWYuU3RyaW5nVmFsdWVcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICByZXR1cm4gYCR7dGhpcy5wcm90b2J1ZlBhY2thZ2VOYW1lKCl9LiR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5wYXJlbnROYW1lLFxuICAgICAgKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIHJldHVybiBgJHt0aGlzLnByb3RvYnVmUGFja2FnZU5hbWUoKX0uJHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLnBhcmVudE5hbWUsXG4gICAgICApfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbikge1xuICAgICAgcmV0dXJuIGAke3RoaXMucHJvdG9idWZQYWNrYWdlTmFtZSgpfS4ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AucGFyZW50TmFtZSxcbiAgICAgICl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BTZWxlY3QgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wVGV4dFxuICAgICkge1xuICAgICAgcmV0dXJuIFwiZ29vZ2xlLnByb3RvYnVmLlN0cmluZ1ZhbHVlXCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIHRocm93IG5ldyBFcnJvcihgVW5rbm93biBwcm9wZXJ0eSB0eXBlIGZvciByZW5kZXJpbmcgcHJvdG9idWYhIEZpeCBtZWApO1xuICAgIH1cbiAgICByZXR1cm4gXCJ1bnJlYWNoYWJsZSFcIjtcbiAgfVxuXG4gIHByb3RvYnVmRGVmaW5pdGlvbkZvclByb3AocHJvcDogUHJvcHMsIGlucHV0TnVtYmVyOiBudW1iZXIpOiBzdHJpbmcge1xuICAgIGxldCByZXBlYXRlZDogc3RyaW5nO1xuICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICByZXBlYXRlZCA9IFwicmVwZWF0ZWQgXCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJlcGVhdGVkID0gXCJcIjtcbiAgICB9XG4gICAgcmV0dXJuIGAke3JlcGVhdGVkfSR7dGhpcy5wcm90b2J1ZlR5cGVGb3JQcm9wKHByb3ApfSAke3NuYWtlQ2FzZShcbiAgICAgIHByb3AubmFtZSxcbiAgICApfSA9ICR7aW5wdXROdW1iZXJ9O2A7XG4gIH1cblxuICBwcm90b2J1Zk1lc3NhZ2VGb3JQcm9wT2JqZWN0KHByb3A6IFByb3BPYmplY3QgfCBQcm9wRW51bSk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuXG4gICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wRW51bSkge1xuICAgICAgbGV0IGVudW1Db3VudCA9IDA7XG4gICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgIGBlbnVtICR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfSB7YCxcbiAgICAgICk7XG4gICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgIGAgICR7Y29uc3RhbnRDYXNlKFxuICAgICAgICAgIHRoaXMucHJvdG9idWZUeXBlRm9yUHJvcChwcm9wKSxcbiAgICAgICAgKX1fVU5LTk9XTiA9ICR7ZW51bUNvdW50fTtgLFxuICAgICAgKTtcbiAgICAgIGZvciAoY29uc3QgdmFyaWFudCBvZiBwcm9wLnZhcmlhbnRzKSB7XG4gICAgICAgIGVudW1Db3VudCA9IGVudW1Db3VudCArIDE7XG4gICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICBgICAke2NvbnN0YW50Q2FzZSh0aGlzLnByb3RvYnVmVHlwZUZvclByb3AocHJvcCkpfV8ke2NvbnN0YW50Q2FzZShcbiAgICAgICAgICAgIHZhcmlhbnQsXG4gICAgICAgICAgKX0gPSAke2VudW1Db3VudH07YCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICAgIHJlc3VsdHMucHVzaChcIn1cIik7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH1cblxuICAgIGZvciAoY29uc3QgYmFnIG9mIHByb3AuYmFnTmFtZXMoKSkge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgZm9yIChjb25zdCBjaGlsZFByb3Agb2YgcHJvcFtiYWddLmF0dHJzKSB7XG4gICAgICAgIGlmIChjaGlsZFByb3AgaW5zdGFuY2VvZiBQcm9wT2JqZWN0IHx8IGNoaWxkUHJvcCBpbnN0YW5jZW9mIFByb3BFbnVtKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMucHJvdG9idWZNZXNzYWdlRm9yUHJvcE9iamVjdChjaGlsZFByb3ApKTtcbiAgICAgICAgfVxuICAgICAgfVxuXG4gICAgICBjb25zdCBtZXNzYWdlTmFtZSA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9YDtcbiAgICAgIHJlc3VsdHMucHVzaChgbWVzc2FnZSAke21lc3NhZ2VOYW1lfSB7YCk7XG5cbiAgICAgIGxldCB1bml2ZXJzYWxCYXNlID0gMDtcbiAgICAgIGxldCBjdXN0b21CYXNlID0gMTAwMDtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIGZvciAoY29uc3QgaW5kZXggaW4gcHJvcFtiYWddLmF0dHJzKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgY29uc3QgcCA9IHByb3BbYmFnXS5hdHRyc1tpbmRleF07XG5cbiAgICAgICAgaWYgKHAudW5pdmVyc2FsKSB7XG4gICAgICAgICAgdW5pdmVyc2FsQmFzZSA9IHVuaXZlcnNhbEJhc2UgKyAxO1xuICAgICAgICAgIHJlc3VsdHMucHVzaChcIiAgXCIgKyB0aGlzLnByb3RvYnVmRGVmaW5pdGlvbkZvclByb3AocCwgdW5pdmVyc2FsQmFzZSkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIGN1c3RvbUJhc2UgPSBjdXN0b21CYXNlICsgMTtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXCIgIFwiICsgdGhpcy5wcm90b2J1ZkRlZmluaXRpb25Gb3JQcm9wKHAsIGN1c3RvbUJhc2UpKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgICAgcmVzdWx0cy5wdXNoKFwifVwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHByb3RvYnVmSW1wb3J0cygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTsgLy8gVGhpcyBjcmVhdGVzIGEgbmV3bGluZSFcbiAgICBjb25zdCByZXN1bHRTZXQgPSB0aGlzLnByb3RvYnVmSW1wb3J0V2FsayhcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0cy5tYXAodiA9PiB2LnJvb3RQcm9wKSxcbiAgICApO1xuICAgIGZvciAoY29uc3QgbGluZSBvZiByZXN1bHRTZXQudmFsdWVzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChgaW1wb3J0IFwiJHtsaW5lfVwiO2ApO1xuICAgIH1cbiAgICBpZiAocmVzdWx0cy5sZW5ndGggPiAwKSB7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCIvLyBObyBJbXBvcnRzXCI7XG4gICAgfVxuICB9XG5cbiAgcHJvdG9idWZJbXBvcnRXYWxrKHByb3BzOiBQcm9wc1tdKTogU2V0PHN0cmluZz4ge1xuICAgIGNvbnN0IHJlc3VsdDogU2V0PHN0cmluZz4gPSBuZXcgU2V0KCk7XG5cbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcHMpIHtcbiAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcImxpbmtcIikge1xuICAgICAgICBjb25zdCBpbXBvcnRQYXRoID0gdGhpcy5wcm90b2J1ZkltcG9ydEZvclByb3AocHJvcCk7XG4gICAgICAgIGlmIChcbiAgICAgICAgICBpbXBvcnRQYXRoICYmXG4gICAgICAgICAgIWltcG9ydFBhdGguc3RhcnRzV2l0aChcbiAgICAgICAgICAgIGBzaS1yZWdpc3RyeS9wcm90by8ke3RoaXMucHJvdG9idWZQYWNrYWdlTmFtZSgpfWAsXG4gICAgICAgICAgKVxuICAgICAgICApIHtcbiAgICAgICAgICByZXN1bHQuYWRkKGltcG9ydFBhdGgpO1xuICAgICAgICB9XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHQuYWRkKFwiZ29vZ2xlL3Byb3RvYnVmL3dyYXBwZXJzLnByb3RvXCIpO1xuICAgICAgfVxuXG4gICAgICBpZiAodGhpcy5yZWN1cnNlS2luZHMuaW5jbHVkZXMocHJvcC5raW5kKCkpKSB7XG4gICAgICAgIGZvciAoY29uc3QgYmFnIG9mIHByb3AuYmFnTmFtZXMoKSkge1xuICAgICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgICBmb3IgKGNvbnN0IGludGVybmFsUHJvcCBvZiBwcm9wW2JhZ10uYXR0cnMpIHtcbiAgICAgICAgICAgIGNvbnN0IG5ld1NldCA9IHRoaXMucHJvdG9idWZJbXBvcnRXYWxrKFtpbnRlcm5hbFByb3BdKTtcbiAgICAgICAgICAgIGZvciAoY29uc3QgaXRlbSBvZiBuZXdTZXQudmFsdWVzKCkpIHtcbiAgICAgICAgICAgICAgcmVzdWx0LmFkZChpdGVtKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlUHJvdG8oKSB7XG4gICAgYXdhaXQgY29kZUZzLndyaXRlQ29kZShcbiAgICAgIGAuL3Byb3RvL3NpLiR7dGhpcy5zeXN0ZW1PYmplY3RzWzBdLnNlcnZpY2VOYW1lfS5wcm90b2AsXG4gICAgICB0aGlzLmdlbmVyYXRlU3RyaW5nKCksXG4gICAgKTtcbiAgfVxuXG4gIGdlbmVyYXRlU3RyaW5nKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9wcm90b2J1Zi9wcm90bycsIHsgZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogdGhpcyxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgfVxufVxuIl19