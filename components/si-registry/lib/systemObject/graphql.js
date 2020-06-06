"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.variablesObjectForProperty = variablesObjectForProperty;
exports.SiGraphql = void 0;

var _taggedTemplateLiteral2 = _interopRequireDefault(require("@babel/runtime/helpers/taggedTemplateLiteral"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _registry = require("../registry");

var _changeCase = require("change-case");

var _graphqlTag = _interopRequireDefault(require("graphql-tag"));

function _templateObject2() {
  var data = (0, _taggedTemplateLiteral2["default"])(["\n      ", "\n    "]);

  _templateObject2 = function _templateObject2() {
    return data;
  };

  return data;
}

function _templateObject() {
  var data = (0, _taggedTemplateLiteral2["default"])(["\n      ", "\n    "]);

  _templateObject = function _templateObject() {
    return data;
  };

  return data;
}

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(n); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

// Second argument is if you want a repeated field
// AKA thePoorlyNamedFunction() :)
function variablesObjectForProperty(prop) {
  var repeated = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : false;

  if (prop.kind() == "text" || prop.kind() == "number" || prop.kind() == "code" || prop.kind() == "enum") {
    if (prop.repeated && repeated) {
      return [];
    } else {
      return "";
    }
  } else if (prop.kind() == "map") {
    if (prop.repeated && repeated) {
      return [];
    } else {
      return [];
    }
  } else if (prop.kind() == "link") {
    var propLink = prop;

    if (prop.repeated && repeated) {
      return [];
    } else {
      // TODO: There might be a bug here, where the name of the prop itself
      // and the name of the linked prop don't match, and so we get the
      // wrong field name if the prop is an object.
      return variablesObjectForProperty(propLink.lookupMyself(), repeated);
    }
  } else if (prop.kind() == "object" || prop.kind() == "method") {
    var propObject = prop;
    var result = {};

    var _iterator = _createForOfIteratorHelper(propObject.properties.attrs),
        _step;

    try {
      for (_iterator.s(); !(_step = _iterator.n()).done;) {
        var field = _step.value;
        var fieldVariables = variablesObjectForProperty(field, repeated);
        result["".concat(field.name)] = fieldVariables;
      }
    } catch (err) {
      _iterator.e(err);
    } finally {
      _iterator.f();
    }

    if (prop.repeated && repeated) {
      return [];
    } else {
      return result;
    }
  }
}

var SiGraphql = /*#__PURE__*/function () {
  function SiGraphql(systemObject) {
    (0, _classCallCheck2["default"])(this, SiGraphql);
    (0, _defineProperty2["default"])(this, "systemObject", void 0);
    this.systemObject = systemObject;
  }

  (0, _createClass2["default"])(SiGraphql, [{
    key: "validateResult",
    value: function validateResult(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var reply = method.reply;
      var lookupName = args.overrideName || "".concat((0, _changeCase.camelCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(args.methodName));
      var result = args.data.data[lookupName];

      var _iterator2 = _createForOfIteratorHelper(reply.properties.attrs),
          _step2;

      try {
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          var field = _step2.value;

          if (field.required && result[field.name] == undefined) {
            throw "response incomplete; missing required field ".concat(field);
          }
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }

      return result;
    }
  }, {
    key: "variablesObject",
    value: function variablesObject(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var request = method.request;
      return variablesObjectForProperty(request, true);
    }
  }, {
    key: "graphqlTypeName",
    value: function graphqlTypeName(prop, inputType) {
      var result = "";

      if (prop.kind() == "object" || prop.kind() == "enum") {
        var request = "";

        if (inputType && prop.kind() != "enum") {
          request = "Request";
        }

        result = "".concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name)).concat(request);
      } else if (prop.kind() == "text" || prop.kind() == "password") {
        if (prop.name == "id") {
          result = "ID";
        } else {
          result = "String";
        }
      } else if (prop.kind() == "number") {
        // @ts-ignore - we don't know about numberKind below
        if (prop.numberKind == "int32") {
          result = "Int";
        } else {
          result = "String";
        }
      } else if (prop.kind() == "link") {
        var linkProp = prop;
        var realProp = linkProp.lookupMyself();
        return this.graphqlTypeName(realProp, inputType);
      }

      if (prop.required) {
        return "".concat(result, "!");
      } else {
        return result;
      }
    }
  }, {
    key: "associationFieldList",
    value: function associationFieldList(associations, systemObject) {
      var associationList = associations && associations[systemObject.typeName];

      if (associationList) {
        var result = [];
        result.push("associations {");

        var _iterator3 = _createForOfIteratorHelper(associationList),
            _step3;

        try {
          for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
            var fieldName = _step3.value;
            var assocObj = systemObject.associations.getByFieldName(fieldName);

            var assocSystem = _registry.registry.get(assocObj.typeName);

            var assocMethod = assocSystem.methods.getEntry(assocObj.methodName);
            result.push("".concat(fieldName, " {"));
            result.push(this.fieldList(assocMethod.reply, associations, assocSystem));
            result.push("}");
          }
        } catch (err) {
          _iterator3.e(err);
        } finally {
          _iterator3.f();
        }

        result.push("}");
        return result.join(" ");
      } else {
        return "";
      }
    }
  }, {
    key: "fieldList",
    value: function fieldList(propObject, associations, systemObjectMemo) {
      var systemObject;

      if (systemObjectMemo) {
        systemObject = systemObjectMemo;
      } else {
        systemObject = this.systemObject;
      }

      var result = [];

      var _iterator4 = _createForOfIteratorHelper(propObject.properties.attrs),
          _step4;

      try {
        for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
          var prop = _step4.value;

          if (prop.hidden || prop.skip) {
            continue;
          }

          result.push("".concat(prop.name)); // without camelCase
          // result.push(`${camelCase(prop.name)}`); // with camelCase

          if (prop.kind() == "object") {
            result.push("{");
            result.push(this.fieldList(prop, undefined, systemObject));
            result.push(this.associationFieldList(associations, systemObject));
            result.push("}");
          }

          if (prop.kind() == "map") {
            result.push("{ key value }");
          } else if (prop.kind() == "link") {
            // @ts-ignore
            var realObj = prop.lookupMyself();

            if (realObj.kind() == "object") {
              result.push("{");
            }

            result.push(this.fieldList(realObj, undefined, systemObject));

            if (realObj.kind() == "object") {
              result.push(this.associationFieldList(associations, systemObject));
              result.push("}");
            }
          }
        }
      } catch (err) {
        _iterator4.e(err);
      } finally {
        _iterator4.f();
      }

      return "".concat(result.join(" "));
    }
  }, {
    key: "query",
    value: function query(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var methodName = args.overrideName || "".concat((0, _changeCase.camelCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(args.methodName));
      var request = method.request;
      var requestVariables = [];
      var inputVariables = [];

      var _iterator5 = _createForOfIteratorHelper(request.properties.attrs),
          _step5;

      try {
        for (_iterator5.s(); !(_step5 = _iterator5.n()).done;) {
          var prop = _step5.value;
          requestVariables.push("$".concat(prop.name, ": ").concat(this.graphqlTypeName(prop, true)) // without camelCase
          // `$${camelCase(prop.name)}: ${this.graphqlTypeName(prop, true)}`, // with camelCase
          );
          inputVariables.push("".concat(prop.name, ": $").concat(prop.name)); // without camelCase
          // inputVariables.push(`${camelCase(prop.name)}: $${camelCase(prop.name)}`); // with camelCase
        }
      } catch (err) {
        _iterator5.e(err);
      } finally {
        _iterator5.f();
      }

      var reply = method.reply;
      var fieldList;

      if (args.overrideFields) {
        fieldList = "".concat(args.overrideFields);
      } else {
        fieldList = this.fieldList(reply, args.associations, this.systemObject);
      }

      var resultString = "query ".concat(methodName, "(").concat(requestVariables.join(", "), ") { ").concat(methodName, "(input: { ").concat(inputVariables.join(", "), " }) { ").concat(fieldList, " } }"); // Log query
      // console.log(`query ${resultString}`);

      return (0, _graphqlTag["default"])(_templateObject(), resultString);
    }
  }, {
    key: "mutation",
    value: function mutation(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var methodName = args.overrideName || "".concat((0, _changeCase.camelCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(args.methodName));
      var request = method.request;
      var requestVariables = [];
      var inputVariables = [];

      var _iterator6 = _createForOfIteratorHelper(request.properties.attrs),
          _step6;

      try {
        for (_iterator6.s(); !(_step6 = _iterator6.n()).done;) {
          var prop = _step6.value;
          requestVariables.push("$".concat(prop.name, ": ").concat(this.graphqlTypeName(prop, true)) // without camelCase
          // `$${camelCase(prop.name)}: ${this.graphqlTypeName(prop, true)}`, // with camelCase
          );
          inputVariables.push("".concat(prop.name, ": $").concat(prop.name)); // without camelCase
          // inputVariables.push(`${camelCase(prop.name)}: $${camelCase(prop.name)}`); // with camelCase
        }
      } catch (err) {
        _iterator6.e(err);
      } finally {
        _iterator6.f();
      }

      var reply = method.reply;
      var fieldList;

      if (args.overrideFields) {
        fieldList = "".concat(args.overrideFields);
      } else {
        fieldList = this.fieldList(reply, args.associations, this.systemObject);
      }

      var resultString = "mutation ".concat(methodName, "(").concat(requestVariables.join(", "), ") { ").concat(methodName, "(input: { ").concat(inputVariables.join(", "), " }) { ").concat(fieldList, " } }");
      console.log(resultString);
      return (0, _graphqlTag["default"])(_templateObject2(), resultString);
    }
  }]);
  return SiGraphql;
}();

exports.SiGraphql = SiGraphql;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJ2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eSIsInByb3AiLCJyZXBlYXRlZCIsImtpbmQiLCJwcm9wTGluayIsImxvb2t1cE15c2VsZiIsInByb3BPYmplY3QiLCJyZXN1bHQiLCJwcm9wZXJ0aWVzIiwiYXR0cnMiLCJmaWVsZCIsImZpZWxkVmFyaWFibGVzIiwibmFtZSIsIlNpR3JhcGhxbCIsInN5c3RlbU9iamVjdCIsImFyZ3MiLCJtZXRob2QiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJtZXRob2ROYW1lIiwicmVwbHkiLCJsb29rdXBOYW1lIiwib3ZlcnJpZGVOYW1lIiwidHlwZU5hbWUiLCJkYXRhIiwicmVxdWlyZWQiLCJ1bmRlZmluZWQiLCJyZXF1ZXN0IiwiaW5wdXRUeXBlIiwicGFyZW50TmFtZSIsIm51bWJlcktpbmQiLCJsaW5rUHJvcCIsInJlYWxQcm9wIiwiZ3JhcGhxbFR5cGVOYW1lIiwiYXNzb2NpYXRpb25zIiwiYXNzb2NpYXRpb25MaXN0IiwicHVzaCIsImZpZWxkTmFtZSIsImFzc29jT2JqIiwiZ2V0QnlGaWVsZE5hbWUiLCJhc3NvY1N5c3RlbSIsInJlZ2lzdHJ5IiwiZ2V0IiwiYXNzb2NNZXRob2QiLCJmaWVsZExpc3QiLCJqb2luIiwic3lzdGVtT2JqZWN0TWVtbyIsImhpZGRlbiIsInNraXAiLCJhc3NvY2lhdGlvbkZpZWxkTGlzdCIsInJlYWxPYmoiLCJyZXF1ZXN0VmFyaWFibGVzIiwiaW5wdXRWYXJpYWJsZXMiLCJvdmVycmlkZUZpZWxkcyIsInJlc3VsdFN0cmluZyIsImdxbCIsImNvbnNvbGUiLCJsb2ciXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUdBOztBQUVBOztBQUNBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBdUJBO0FBQ0E7QUFDTyxTQUFTQSwwQkFBVCxDQUFvQ0MsSUFBcEMsRUFBd0U7QUFBQSxNQUF2QkMsUUFBdUIsdUVBQVosS0FBWTs7QUFDN0UsTUFDRUQsSUFBSSxDQUFDRSxJQUFMLE1BQWUsTUFBZixJQUNBRixJQUFJLENBQUNFLElBQUwsTUFBZSxRQURmLElBRUFGLElBQUksQ0FBQ0UsSUFBTCxNQUFlLE1BRmYsSUFHQUYsSUFBSSxDQUFDRSxJQUFMLE1BQWUsTUFKakIsRUFLRTtBQUNBLFFBQUlGLElBQUksQ0FBQ0MsUUFBTCxJQUFpQkEsUUFBckIsRUFBK0I7QUFDN0IsYUFBTyxFQUFQO0FBQ0QsS0FGRCxNQUVPO0FBQ0wsYUFBTyxFQUFQO0FBQ0Q7QUFDRixHQVhELE1BV08sSUFBSUQsSUFBSSxDQUFDRSxJQUFMLE1BQWUsS0FBbkIsRUFBMEI7QUFDL0IsUUFBSUYsSUFBSSxDQUFDQyxRQUFMLElBQWlCQSxRQUFyQixFQUErQjtBQUM3QixhQUFPLEVBQVA7QUFDRCxLQUZELE1BRU87QUFDTCxhQUFPLEVBQVA7QUFDRDtBQUNGLEdBTk0sTUFNQSxJQUFJRCxJQUFJLENBQUNFLElBQUwsTUFBZSxNQUFuQixFQUEyQjtBQUNoQyxRQUFNQyxRQUFRLEdBQUdILElBQWpCOztBQUNBLFFBQUlBLElBQUksQ0FBQ0MsUUFBTCxJQUFpQkEsUUFBckIsRUFBK0I7QUFDN0IsYUFBTyxFQUFQO0FBQ0QsS0FGRCxNQUVPO0FBQ0w7QUFDQTtBQUNBO0FBQ0EsYUFBT0YsMEJBQTBCLENBQUNJLFFBQVEsQ0FBQ0MsWUFBVCxFQUFELEVBQTBCSCxRQUExQixDQUFqQztBQUNEO0FBQ0YsR0FWTSxNQVVBLElBQUlELElBQUksQ0FBQ0UsSUFBTCxNQUFlLFFBQWYsSUFBMkJGLElBQUksQ0FBQ0UsSUFBTCxNQUFlLFFBQTlDLEVBQXdEO0FBQzdELFFBQU1HLFVBQVUsR0FBR0wsSUFBbkI7QUFDQSxRQUFNTSxNQUErQixHQUFHLEVBQXhDOztBQUY2RCwrQ0FHekNELFVBQVUsQ0FBQ0UsVUFBWCxDQUFzQkMsS0FIbUI7QUFBQTs7QUFBQTtBQUc3RCwwREFBaUQ7QUFBQSxZQUF0Q0MsS0FBc0M7QUFDL0MsWUFBTUMsY0FBYyxHQUFHWCwwQkFBMEIsQ0FBQ1UsS0FBRCxFQUFRUixRQUFSLENBQWpEO0FBQ0FLLFFBQUFBLE1BQU0sV0FBSUcsS0FBSyxDQUFDRSxJQUFWLEVBQU4sR0FBMEJELGNBQTFCO0FBQ0Q7QUFONEQ7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFPN0QsUUFBSVYsSUFBSSxDQUFDQyxRQUFMLElBQWlCQSxRQUFyQixFQUErQjtBQUM3QixhQUFPLEVBQVA7QUFDRCxLQUZELE1BRU87QUFDTCxhQUFPSyxNQUFQO0FBQ0Q7QUFDRjtBQUNGOztJQUVZTSxTO0FBR1gscUJBQVlDLFlBQVosRUFBcUQ7QUFBQTtBQUFBO0FBQ25ELFNBQUtBLFlBQUwsR0FBb0JBLFlBQXBCO0FBQ0Q7Ozs7bUNBRWNDLEksRUFBK0M7QUFDNUQsVUFBTUMsTUFBTSxHQUFHLEtBQUtGLFlBQUwsQ0FBa0JHLE9BQWxCLENBQTBCQyxRQUExQixDQUNiSCxJQUFJLENBQUNJLFVBRFEsQ0FBZjtBQUdBLFVBQU1DLEtBQUssR0FBR0osTUFBTSxDQUFDSSxLQUFyQjtBQUNBLFVBQU1DLFVBQVUsR0FDZE4sSUFBSSxDQUFDTyxZQUFMLGNBQ0csMkJBQVUsS0FBS1IsWUFBTCxDQUFrQlMsUUFBNUIsQ0FESCxTQUMyQyw0QkFBV1IsSUFBSSxDQUFDSSxVQUFoQixDQUQzQyxDQURGO0FBR0EsVUFBTVosTUFBTSxHQUFHUSxJQUFJLENBQUNTLElBQUwsQ0FBVUEsSUFBVixDQUFlSCxVQUFmLENBQWY7O0FBUjRELGtEQVN4Q0QsS0FBSyxDQUFDWixVQUFOLENBQWlCQyxLQVR1QjtBQUFBOztBQUFBO0FBUzVELCtEQUE0QztBQUFBLGNBQWpDQyxLQUFpQzs7QUFDMUMsY0FBSUEsS0FBSyxDQUFDZSxRQUFOLElBQWtCbEIsTUFBTSxDQUFDRyxLQUFLLENBQUNFLElBQVAsQ0FBTixJQUFzQmMsU0FBNUMsRUFBdUQ7QUFDckQsd0VBQXFEaEIsS0FBckQ7QUFDRDtBQUNGO0FBYjJEO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBYzVELGFBQU9ILE1BQVA7QUFDRDs7O29DQUVlUSxJLEVBQWdEO0FBQzlELFVBQU1DLE1BQU0sR0FBRyxLQUFLRixZQUFMLENBQWtCRyxPQUFsQixDQUEwQkMsUUFBMUIsQ0FDYkgsSUFBSSxDQUFDSSxVQURRLENBQWY7QUFHQSxVQUFNUSxPQUFPLEdBQUdYLE1BQU0sQ0FBQ1csT0FBdkI7QUFDQSxhQUFPM0IsMEJBQTBCLENBQUMyQixPQUFELEVBQVUsSUFBVixDQUFqQztBQUNEOzs7b0NBRWUxQixJLEVBQWEyQixTLEVBQTZCO0FBQ3hELFVBQUlyQixNQUFNLEdBQUcsRUFBYjs7QUFDQSxVQUFJTixJQUFJLENBQUNFLElBQUwsTUFBZSxRQUFmLElBQTJCRixJQUFJLENBQUNFLElBQUwsTUFBZSxNQUE5QyxFQUFzRDtBQUNwRCxZQUFJd0IsT0FBTyxHQUFHLEVBQWQ7O0FBQ0EsWUFBSUMsU0FBUyxJQUFJM0IsSUFBSSxDQUFDRSxJQUFMLE1BQWUsTUFBaEMsRUFBd0M7QUFDdEN3QixVQUFBQSxPQUFPLEdBQUcsU0FBVjtBQUNEOztBQUNEcEIsUUFBQUEsTUFBTSxhQUFNLDRCQUFXTixJQUFJLENBQUM0QixVQUFoQixDQUFOLFNBQW9DLDRCQUN4QzVCLElBQUksQ0FBQ1csSUFEbUMsQ0FBcEMsU0FFRmUsT0FGRSxDQUFOO0FBR0QsT0FSRCxNQVFPLElBQUkxQixJQUFJLENBQUNFLElBQUwsTUFBZSxNQUFmLElBQXlCRixJQUFJLENBQUNFLElBQUwsTUFBZSxVQUE1QyxFQUF3RDtBQUM3RCxZQUFJRixJQUFJLENBQUNXLElBQUwsSUFBYSxJQUFqQixFQUF1QjtBQUNyQkwsVUFBQUEsTUFBTSxHQUFHLElBQVQ7QUFDRCxTQUZELE1BRU87QUFDTEEsVUFBQUEsTUFBTSxHQUFHLFFBQVQ7QUFDRDtBQUNGLE9BTk0sTUFNQSxJQUFJTixJQUFJLENBQUNFLElBQUwsTUFBZSxRQUFuQixFQUE2QjtBQUNoQztBQUNBLFlBQUlGLElBQUksQ0FBQzZCLFVBQUwsSUFBbUIsT0FBdkIsRUFBZ0M7QUFDOUJ2QixVQUFBQSxNQUFNLEdBQUcsS0FBVDtBQUNELFNBRkQsTUFFTztBQUNQQSxVQUFBQSxNQUFNLEdBQUcsUUFBVDtBQUNIO0FBQ0EsT0FQTSxNQU9BLElBQUlOLElBQUksQ0FBQ0UsSUFBTCxNQUFlLE1BQW5CLEVBQTJCO0FBQ2hDLFlBQU00QixRQUFRLEdBQUc5QixJQUFqQjtBQUNBLFlBQU0rQixRQUFRLEdBQUdELFFBQVEsQ0FBQzFCLFlBQVQsRUFBakI7QUFDQSxlQUFPLEtBQUs0QixlQUFMLENBQXFCRCxRQUFyQixFQUErQkosU0FBL0IsQ0FBUDtBQUNEOztBQUNELFVBQUkzQixJQUFJLENBQUN3QixRQUFULEVBQW1CO0FBQ2pCLHlCQUFVbEIsTUFBVjtBQUNELE9BRkQsTUFFTztBQUNMLGVBQU9BLE1BQVA7QUFDRDtBQUNGOzs7eUNBR0MyQixZLEVBQ0FwQixZLEVBQ1E7QUFDUixVQUFNcUIsZUFBZSxHQUFHRCxZQUFZLElBQUlBLFlBQVksQ0FBQ3BCLFlBQVksQ0FBQ1MsUUFBZCxDQUFwRDs7QUFDQSxVQUFJWSxlQUFKLEVBQXFCO0FBQ25CLFlBQU01QixNQUFnQixHQUFHLEVBQXpCO0FBQ0FBLFFBQUFBLE1BQU0sQ0FBQzZCLElBQVAsQ0FBWSxnQkFBWjs7QUFGbUIsb0RBR0tELGVBSEw7QUFBQTs7QUFBQTtBQUduQixpRUFBeUM7QUFBQSxnQkFBOUJFLFNBQThCO0FBQ3ZDLGdCQUFNQyxRQUFRLEdBQUd4QixZQUFZLENBQUNvQixZQUFiLENBQTBCSyxjQUExQixDQUF5Q0YsU0FBekMsQ0FBakI7O0FBQ0EsZ0JBQU1HLFdBQVcsR0FBR0MsbUJBQVNDLEdBQVQsQ0FBYUosUUFBUSxDQUFDZixRQUF0QixDQUFwQjs7QUFDQSxnQkFBTW9CLFdBQVcsR0FBR0gsV0FBVyxDQUFDdkIsT0FBWixDQUFvQkMsUUFBcEIsQ0FDbEJvQixRQUFRLENBQUNuQixVQURTLENBQXBCO0FBSUFaLFlBQUFBLE1BQU0sQ0FBQzZCLElBQVAsV0FBZUMsU0FBZjtBQUNBOUIsWUFBQUEsTUFBTSxDQUFDNkIsSUFBUCxDQUNFLEtBQUtRLFNBQUwsQ0FBZUQsV0FBVyxDQUFDdkIsS0FBM0IsRUFBa0NjLFlBQWxDLEVBQWdETSxXQUFoRCxDQURGO0FBR0FqQyxZQUFBQSxNQUFNLENBQUM2QixJQUFQO0FBQ0Q7QUFma0I7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFnQm5CN0IsUUFBQUEsTUFBTSxDQUFDNkIsSUFBUCxDQUFZLEdBQVo7QUFDQSxlQUFPN0IsTUFBTSxDQUFDc0MsSUFBUCxDQUFZLEdBQVosQ0FBUDtBQUNELE9BbEJELE1Ba0JPO0FBQ0wsZUFBTyxFQUFQO0FBQ0Q7QUFDRjs7OzhCQUdDdkMsVSxFQUNBNEIsWSxFQUNBWSxnQixFQUNRO0FBQ1IsVUFBSWhDLFlBQUo7O0FBQ0EsVUFBSWdDLGdCQUFKLEVBQXNCO0FBQ3BCaEMsUUFBQUEsWUFBWSxHQUFHZ0MsZ0JBQWY7QUFDRCxPQUZELE1BRU87QUFDTGhDLFFBQUFBLFlBQVksR0FBRyxLQUFLQSxZQUFwQjtBQUNEOztBQUNELFVBQU1QLE1BQWdCLEdBQUcsRUFBekI7O0FBUFEsa0RBUVdELFVBQVUsQ0FBQ0UsVUFBWCxDQUFzQkMsS0FSakM7QUFBQTs7QUFBQTtBQVFSLCtEQUFnRDtBQUFBLGNBQXJDUixJQUFxQzs7QUFDOUMsY0FBSUEsSUFBSSxDQUFDOEMsTUFBTCxJQUFlOUMsSUFBSSxDQUFDK0MsSUFBeEIsRUFBOEI7QUFDNUI7QUFDRDs7QUFDRHpDLFVBQUFBLE1BQU0sQ0FBQzZCLElBQVAsV0FBZW5DLElBQUksQ0FBQ1csSUFBcEIsR0FKOEMsQ0FJakI7QUFDN0I7O0FBQ0EsY0FBSVgsSUFBSSxDQUFDRSxJQUFMLE1BQWUsUUFBbkIsRUFBNkI7QUFDM0JJLFlBQUFBLE1BQU0sQ0FBQzZCLElBQVAsQ0FBWSxHQUFaO0FBQ0E3QixZQUFBQSxNQUFNLENBQUM2QixJQUFQLENBQ0UsS0FBS1EsU0FBTCxDQUFlM0MsSUFBZixFQUFtQ3lCLFNBQW5DLEVBQThDWixZQUE5QyxDQURGO0FBR0FQLFlBQUFBLE1BQU0sQ0FBQzZCLElBQVAsQ0FBWSxLQUFLYSxvQkFBTCxDQUEwQmYsWUFBMUIsRUFBd0NwQixZQUF4QyxDQUFaO0FBQ0FQLFlBQUFBLE1BQU0sQ0FBQzZCLElBQVAsQ0FBWSxHQUFaO0FBQ0Q7O0FBQ0QsY0FBSW5DLElBQUksQ0FBQ0UsSUFBTCxNQUFlLEtBQW5CLEVBQTBCO0FBQ3hCSSxZQUFBQSxNQUFNLENBQUM2QixJQUFQLENBQVksZUFBWjtBQUNELFdBRkQsTUFFTyxJQUFJbkMsSUFBSSxDQUFDRSxJQUFMLE1BQWUsTUFBbkIsRUFBMkI7QUFDaEM7QUFDQSxnQkFBTStDLE9BQU8sR0FBR2pELElBQUksQ0FBQ0ksWUFBTCxFQUFoQjs7QUFDQSxnQkFBSTZDLE9BQU8sQ0FBQy9DLElBQVIsTUFBa0IsUUFBdEIsRUFBZ0M7QUFDOUJJLGNBQUFBLE1BQU0sQ0FBQzZCLElBQVAsQ0FBWSxHQUFaO0FBQ0Q7O0FBQ0Q3QixZQUFBQSxNQUFNLENBQUM2QixJQUFQLENBQ0UsS0FBS1EsU0FBTCxDQUFlTSxPQUFmLEVBQXNDeEIsU0FBdEMsRUFBaURaLFlBQWpELENBREY7O0FBR0EsZ0JBQUlvQyxPQUFPLENBQUMvQyxJQUFSLE1BQWtCLFFBQXRCLEVBQWdDO0FBQzlCSSxjQUFBQSxNQUFNLENBQUM2QixJQUFQLENBQVksS0FBS2Esb0JBQUwsQ0FBMEJmLFlBQTFCLEVBQXdDcEIsWUFBeEMsQ0FBWjtBQUNBUCxjQUFBQSxNQUFNLENBQUM2QixJQUFQLENBQVksR0FBWjtBQUNEO0FBQ0Y7QUFDRjtBQXRDTztBQUFBO0FBQUE7QUFBQTtBQUFBOztBQXVDUix1QkFBVTdCLE1BQU0sQ0FBQ3NDLElBQVAsQ0FBWSxHQUFaLENBQVY7QUFDRDs7OzBCQUVLOUIsSSxFQUErQjtBQUNuQyxVQUFNQyxNQUFNLEdBQUcsS0FBS0YsWUFBTCxDQUFrQkcsT0FBbEIsQ0FBMEJDLFFBQTFCLENBQ2JILElBQUksQ0FBQ0ksVUFEUSxDQUFmO0FBR0EsVUFBTUEsVUFBVSxHQUNkSixJQUFJLENBQUNPLFlBQUwsY0FDRywyQkFBVSxLQUFLUixZQUFMLENBQWtCUyxRQUE1QixDQURILFNBQzJDLDRCQUFXUixJQUFJLENBQUNJLFVBQWhCLENBRDNDLENBREY7QUFJQSxVQUFNUSxPQUFPLEdBQUdYLE1BQU0sQ0FBQ1csT0FBdkI7QUFDQSxVQUFNd0IsZ0JBQWdCLEdBQUcsRUFBekI7QUFDQSxVQUFNQyxjQUFjLEdBQUcsRUFBdkI7O0FBVm1DLGtEQVdoQnpCLE9BQU8sQ0FBQ25CLFVBQVIsQ0FBbUJDLEtBWEg7QUFBQTs7QUFBQTtBQVduQywrREFBNkM7QUFBQSxjQUFsQ1IsSUFBa0M7QUFDM0NrRCxVQUFBQSxnQkFBZ0IsQ0FBQ2YsSUFBakIsWUFDTW5DLElBQUksQ0FBQ1csSUFEWCxlQUNvQixLQUFLcUIsZUFBTCxDQUFxQmhDLElBQXJCLEVBQTJCLElBQTNCLENBRHBCLEVBQ3dEO0FBQ3REO0FBRkY7QUFJQW1ELFVBQUFBLGNBQWMsQ0FBQ2hCLElBQWYsV0FBdUJuQyxJQUFJLENBQUNXLElBQTVCLGdCQUFzQ1gsSUFBSSxDQUFDVyxJQUEzQyxHQUwyQyxDQUtTO0FBQ3BEO0FBQ0Q7QUFsQmtDO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBb0JuQyxVQUFNUSxLQUFLLEdBQUdKLE1BQU0sQ0FBQ0ksS0FBckI7QUFDQSxVQUFJd0IsU0FBSjs7QUFDQSxVQUFJN0IsSUFBSSxDQUFDc0MsY0FBVCxFQUF5QjtBQUN2QlQsUUFBQUEsU0FBUyxhQUFNN0IsSUFBSSxDQUFDc0MsY0FBWCxDQUFUO0FBQ0QsT0FGRCxNQUVPO0FBQ0xULFFBQUFBLFNBQVMsR0FBRyxLQUFLQSxTQUFMLENBQWV4QixLQUFmLEVBQXNCTCxJQUFJLENBQUNtQixZQUEzQixFQUF5QyxLQUFLcEIsWUFBOUMsQ0FBWjtBQUNEOztBQUVELFVBQU13QyxZQUFZLG1CQUFZbkMsVUFBWixjQUEwQmdDLGdCQUFnQixDQUFDTixJQUFqQixDQUMxQyxJQUQwQyxDQUExQixpQkFFVjFCLFVBRlUsdUJBRWFpQyxjQUFjLENBQUNQLElBQWYsQ0FDN0IsSUFENkIsQ0FGYixtQkFJUkQsU0FKUSxTQUFsQixDQTVCbUMsQ0FrQ25DO0FBQ0E7O0FBRUEsaUJBQU9XLHNCQUFQLHFCQUNJRCxZQURKO0FBR0Q7Ozs2QkFFUXZDLEksRUFBK0I7QUFDdEMsVUFBTUMsTUFBTSxHQUFHLEtBQUtGLFlBQUwsQ0FBa0JHLE9BQWxCLENBQTBCQyxRQUExQixDQUNiSCxJQUFJLENBQUNJLFVBRFEsQ0FBZjtBQUdBLFVBQU1BLFVBQVUsR0FDZEosSUFBSSxDQUFDTyxZQUFMLGNBQ0csMkJBQVUsS0FBS1IsWUFBTCxDQUFrQlMsUUFBNUIsQ0FESCxTQUMyQyw0QkFBV1IsSUFBSSxDQUFDSSxVQUFoQixDQUQzQyxDQURGO0FBSUEsVUFBTVEsT0FBTyxHQUFHWCxNQUFNLENBQUNXLE9BQXZCO0FBQ0EsVUFBTXdCLGdCQUFnQixHQUFHLEVBQXpCO0FBQ0EsVUFBTUMsY0FBYyxHQUFHLEVBQXZCOztBQVZzQyxrREFXbkJ6QixPQUFPLENBQUNuQixVQUFSLENBQW1CQyxLQVhBO0FBQUE7O0FBQUE7QUFXdEMsK0RBQTZDO0FBQUEsY0FBbENSLElBQWtDO0FBQzNDa0QsVUFBQUEsZ0JBQWdCLENBQUNmLElBQWpCLFlBQ01uQyxJQUFJLENBQUNXLElBRFgsZUFDb0IsS0FBS3FCLGVBQUwsQ0FBcUJoQyxJQUFyQixFQUEyQixJQUEzQixDQURwQixFQUN3RDtBQUN0RDtBQUZGO0FBSUFtRCxVQUFBQSxjQUFjLENBQUNoQixJQUFmLFdBQXVCbkMsSUFBSSxDQUFDVyxJQUE1QixnQkFBc0NYLElBQUksQ0FBQ1csSUFBM0MsR0FMMkMsQ0FLUztBQUNwRDtBQUNEO0FBbEJxQztBQUFBO0FBQUE7QUFBQTtBQUFBOztBQW9CdEMsVUFBTVEsS0FBSyxHQUFHSixNQUFNLENBQUNJLEtBQXJCO0FBQ0EsVUFBSXdCLFNBQUo7O0FBQ0EsVUFBSTdCLElBQUksQ0FBQ3NDLGNBQVQsRUFBeUI7QUFDdkJULFFBQUFBLFNBQVMsYUFBTTdCLElBQUksQ0FBQ3NDLGNBQVgsQ0FBVDtBQUNELE9BRkQsTUFFTztBQUNMVCxRQUFBQSxTQUFTLEdBQUcsS0FBS0EsU0FBTCxDQUFleEIsS0FBZixFQUFzQkwsSUFBSSxDQUFDbUIsWUFBM0IsRUFBeUMsS0FBS3BCLFlBQTlDLENBQVo7QUFDRDs7QUFFRCxVQUFNd0MsWUFBWSxzQkFBZW5DLFVBQWYsY0FBNkJnQyxnQkFBZ0IsQ0FBQ04sSUFBakIsQ0FDN0MsSUFENkMsQ0FBN0IsaUJBRVYxQixVQUZVLHVCQUVhaUMsY0FBYyxDQUFDUCxJQUFmLENBQzdCLElBRDZCLENBRmIsbUJBSVJELFNBSlEsU0FBbEI7QUFLQVksTUFBQUEsT0FBTyxDQUFDQyxHQUFSLENBQVlILFlBQVo7QUFDQSxpQkFBT0Msc0JBQVAsc0JBQ0lELFlBREo7QUFHRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3BNZXRob2QsIFByb3BPYmplY3QsIFByb3BzIH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5pbXBvcnQgeyBQcm9wTGluayB9IGZyb20gXCIuLi9wcm9wL2xpbmtcIjtcbmltcG9ydCB7IE9iamVjdFR5cGVzIH0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcblxuaW1wb3J0IHsgcGFzY2FsQ2FzZSwgY2FtZWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5pbXBvcnQgZ3FsIGZyb20gXCJncmFwaHFsLXRhZ1wiO1xuaW1wb3J0IHsgRG9jdW1lbnROb2RlIH0gZnJvbSBcImdyYXBocWxcIjtcbmltcG9ydCB7IEFzc29jaWF0aW9uIH0gZnJvbSBcIi4vYXNzb2NpYXRpb25zXCI7XG5cbmV4cG9ydCBpbnRlcmZhY2UgUXVlcnlBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xuICBvdmVycmlkZU5hbWU/OiBzdHJpbmc7XG4gIG92ZXJyaWRlRmllbGRzPzogc3RyaW5nO1xuICBhc3NvY2lhdGlvbnM/OiB7XG4gICAgW2tleTogc3RyaW5nXTogc3RyaW5nW107XG4gIH07XG59XG5cbmV4cG9ydCBpbnRlcmZhY2UgVmFyaWFibGVzT2JqZWN0QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbn1cblxuZXhwb3J0IGludGVyZmFjZSBWYWxpZGF0ZVJlc3VsdEFyZ3Mge1xuICBtZXRob2ROYW1lOiBzdHJpbmc7XG4gIGRhdGE6IFJlY29yZDxzdHJpbmcsIGFueT47XG4gIG92ZXJyaWRlTmFtZT86IHN0cmluZztcbn1cblxuLy8gU2Vjb25kIGFyZ3VtZW50IGlzIGlmIHlvdSB3YW50IGEgcmVwZWF0ZWQgZmllbGRcbi8vIEFLQSB0aGVQb29ybHlOYW1lZEZ1bmN0aW9uKCkgOilcbmV4cG9ydCBmdW5jdGlvbiB2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eShwcm9wOiBQcm9wcywgcmVwZWF0ZWQgPSBmYWxzZSk6IGFueSB7XG4gIGlmIChcbiAgICBwcm9wLmtpbmQoKSA9PSBcInRleHRcIiB8fFxuICAgIHByb3Aua2luZCgpID09IFwibnVtYmVyXCIgfHxcbiAgICBwcm9wLmtpbmQoKSA9PSBcImNvZGVcIiB8fFxuICAgIHByb3Aua2luZCgpID09IFwiZW51bVwiXG4gICkge1xuICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlwiO1xuICAgIH1cbiAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm1hcFwiKSB7XG4gICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH1cbiAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcImxpbmtcIikge1xuICAgIGNvbnN0IHByb3BMaW5rID0gcHJvcCBhcyBQcm9wTGluaztcbiAgICBpZiAocHJvcC5yZXBlYXRlZCAmJiByZXBlYXRlZCkge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH0gZWxzZSB7XG4gICAgICAvLyBUT0RPOiBUaGVyZSBtaWdodCBiZSBhIGJ1ZyBoZXJlLCB3aGVyZSB0aGUgbmFtZSBvZiB0aGUgcHJvcCBpdHNlbGZcbiAgICAgIC8vIGFuZCB0aGUgbmFtZSBvZiB0aGUgbGlua2VkIHByb3AgZG9uJ3QgbWF0Y2gsIGFuZCBzbyB3ZSBnZXQgdGhlXG4gICAgICAvLyB3cm9uZyBmaWVsZCBuYW1lIGlmIHRoZSBwcm9wIGlzIGFuIG9iamVjdC5cbiAgICAgIHJldHVybiB2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eShwcm9wTGluay5sb29rdXBNeXNlbGYoKSwgcmVwZWF0ZWQpO1xuICAgIH1cbiAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm9iamVjdFwiIHx8IHByb3Aua2luZCgpID09IFwibWV0aG9kXCIpIHtcbiAgICBjb25zdCBwcm9wT2JqZWN0ID0gcHJvcCBhcyBQcm9wT2JqZWN0O1xuICAgIGNvbnN0IHJlc3VsdDogUmVjb3JkPHN0cmluZywgdW5rbm93bj4gPSB7fTtcbiAgICBmb3IgKGNvbnN0IGZpZWxkIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgY29uc3QgZmllbGRWYXJpYWJsZXMgPSB2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eShmaWVsZCwgcmVwZWF0ZWQpO1xuICAgICAgcmVzdWx0W2Ake2ZpZWxkLm5hbWV9YF0gPSBmaWVsZFZhcmlhYmxlcztcbiAgICB9XG4gICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHJlc3VsdDtcbiAgICB9XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFNpR3JhcGhxbCB7XG4gIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBTaUdyYXBocWxbXCJzeXN0ZW1PYmplY3RcIl0pIHtcbiAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbiAgfVxuXG4gIHZhbGlkYXRlUmVzdWx0KGFyZ3M6IFZhbGlkYXRlUmVzdWx0QXJncyk6IFJlY29yZDxzdHJpbmcsIGFueT4ge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGNvbnN0IGxvb2t1cE5hbWUgPVxuICAgICAgYXJncy5vdmVycmlkZU5hbWUgfHxcbiAgICAgIGAke2NhbWVsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9JHtwYXNjYWxDYXNlKGFyZ3MubWV0aG9kTmFtZSl9YDtcbiAgICBjb25zdCByZXN1bHQgPSBhcmdzLmRhdGEuZGF0YVtsb29rdXBOYW1lXTtcbiAgICBmb3IgKGNvbnN0IGZpZWxkIG9mIHJlcGx5LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChmaWVsZC5yZXF1aXJlZCAmJiByZXN1bHRbZmllbGQubmFtZV0gPT0gdW5kZWZpbmVkKSB7XG4gICAgICAgIHRocm93IGByZXNwb25zZSBpbmNvbXBsZXRlOyBtaXNzaW5nIHJlcXVpcmVkIGZpZWxkICR7ZmllbGR9YDtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIHZhcmlhYmxlc09iamVjdChhcmdzOiBWYXJpYWJsZXNPYmplY3RBcmdzKTogUmVjb3JkPHN0cmluZywgYW55PiB7XG4gICAgY29uc3QgbWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSxcbiAgICApIGFzIFByb3BNZXRob2Q7XG4gICAgY29uc3QgcmVxdWVzdCA9IG1ldGhvZC5yZXF1ZXN0O1xuICAgIHJldHVybiB2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eShyZXF1ZXN0LCB0cnVlKTtcbiAgfVxuXG4gIGdyYXBocWxUeXBlTmFtZShwcm9wOiBQcm9wcywgaW5wdXRUeXBlPzogYm9vbGVhbik6IHN0cmluZyB7XG4gICAgbGV0IHJlc3VsdCA9IFwiXCI7XG4gICAgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIgfHwgcHJvcC5raW5kKCkgPT0gXCJlbnVtXCIpIHtcbiAgICAgIGxldCByZXF1ZXN0ID0gXCJcIjtcbiAgICAgIGlmIChpbnB1dFR5cGUgJiYgcHJvcC5raW5kKCkgIT0gXCJlbnVtXCIpIHtcbiAgICAgICAgcmVxdWVzdCA9IFwiUmVxdWVzdFwiO1xuICAgICAgfVxuICAgICAgcmVzdWx0ID0gYCR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX0ke3JlcXVlc3R9YDtcbiAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwidGV4dFwiIHx8IHByb3Aua2luZCgpID09IFwicGFzc3dvcmRcIikge1xuICAgICAgaWYgKHByb3AubmFtZSA9PSBcImlkXCIpIHtcbiAgICAgICAgcmVzdWx0ID0gXCJJRFwiO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0ID0gXCJTdHJpbmdcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibnVtYmVyXCIpIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvbid0IGtub3cgYWJvdXQgbnVtYmVyS2luZCBiZWxvd1xuICAgICAgICBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50MzJcIikge1xuICAgICAgICAgIHJlc3VsdCA9IFwiSW50XCI7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdCA9IFwiU3RyaW5nXCI7XG4gICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgIGNvbnN0IGxpbmtQcm9wID0gcHJvcCBhcyBQcm9wTGluaztcbiAgICAgIGNvbnN0IHJlYWxQcm9wID0gbGlua1Byb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICByZXR1cm4gdGhpcy5ncmFwaHFsVHlwZU5hbWUocmVhbFByb3AsIGlucHV0VHlwZSk7XG4gICAgfVxuICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4gICAgICByZXR1cm4gYCR7cmVzdWx0fSFgO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gcmVzdWx0O1xuICAgIH1cbiAgfVxuXG4gIGFzc29jaWF0aW9uRmllbGRMaXN0KFxuICAgIGFzc29jaWF0aW9uczogUXVlcnlBcmdzW1wiYXNzb2NpYXRpb25zXCJdLFxuICAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgYXNzb2NpYXRpb25MaXN0ID0gYXNzb2NpYXRpb25zICYmIGFzc29jaWF0aW9uc1tzeXN0ZW1PYmplY3QudHlwZU5hbWVdO1xuICAgIGlmIChhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYXNzb2NpYXRpb25zIHtcIik7XG4gICAgICBmb3IgKGNvbnN0IGZpZWxkTmFtZSBvZiBhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgICAgY29uc3QgYXNzb2NPYmogPSBzeXN0ZW1PYmplY3QuYXNzb2NpYXRpb25zLmdldEJ5RmllbGROYW1lKGZpZWxkTmFtZSk7XG4gICAgICAgIGNvbnN0IGFzc29jU3lzdGVtID0gcmVnaXN0cnkuZ2V0KGFzc29jT2JqLnR5cGVOYW1lKTtcbiAgICAgICAgY29uc3QgYXNzb2NNZXRob2QgPSBhc3NvY1N5c3RlbS5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgICAgIGFzc29jT2JqLm1ldGhvZE5hbWUsXG4gICAgICAgICkgYXMgUHJvcE1ldGhvZDtcblxuICAgICAgICByZXN1bHQucHVzaChgJHtmaWVsZE5hbWV9IHtgKTtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QoYXNzb2NNZXRob2QucmVwbHksIGFzc29jaWF0aW9ucywgYXNzb2NTeXN0ZW0pLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHQucHVzaChgfWApO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goXCJ9XCIpO1xuICAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiIFwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9XG5cbiAgZmllbGRMaXN0KFxuICAgIHByb3BPYmplY3Q6IFByb3BPYmplY3QsXG4gICAgYXNzb2NpYXRpb25zOiBRdWVyeUFyZ3NbXCJhc3NvY2lhdGlvbnNcIl0sXG4gICAgc3lzdGVtT2JqZWN0TWVtbzogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgbGV0IHN5c3RlbU9iamVjdDtcbiAgICBpZiAoc3lzdGVtT2JqZWN0TWVtbykge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0TWVtbztcbiAgICB9IGVsc2Uge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gdGhpcy5zeXN0ZW1PYmplY3Q7XG4gICAgfVxuICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4gfHwgcHJvcC5za2lwKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goYCR7cHJvcC5uYW1lfWApOyAvLyB3aXRob3V0IGNhbWVsQ2FzZVxuICAgICAgLy8gcmVzdWx0LnB1c2goYCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9YCk7IC8vIHdpdGggY2FtZWxDYXNlXG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICByZXN1bHQucHVzaChcIntcIik7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KHByb3AgYXMgUHJvcE9iamVjdCwgdW5kZWZpbmVkLCBzeXN0ZW1PYmplY3QpLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHQucHVzaCh0aGlzLmFzc29jaWF0aW9uRmllbGRMaXN0KGFzc29jaWF0aW9ucywgc3lzdGVtT2JqZWN0KSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcIm1hcFwiKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKFwieyBrZXkgdmFsdWUgfVwiKTtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgICBjb25zdCByZWFsT2JqID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgICAgaWYgKHJlYWxPYmoua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChcIntcIik7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QocmVhbE9iaiBhcyBQcm9wT2JqZWN0LCB1bmRlZmluZWQsIHN5c3RlbU9iamVjdCksXG4gICAgICAgICk7XG4gICAgICAgIGlmIChyZWFsT2JqLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2godGhpcy5hc3NvY2lhdGlvbkZpZWxkTGlzdChhc3NvY2lhdGlvbnMsIHN5c3RlbU9iamVjdCkpO1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gYCR7cmVzdWx0LmpvaW4oXCIgXCIpfWA7XG4gIH1cblxuICBxdWVyeShhcmdzOiBRdWVyeUFyZ3MpOiBEb2N1bWVudE5vZGUge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IG1ldGhvZE5hbWUgPVxuICAgICAgYXJncy5vdmVycmlkZU5hbWUgfHxcbiAgICAgIGAke2NhbWVsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9JHtwYXNjYWxDYXNlKGFyZ3MubWV0aG9kTmFtZSl9YDtcblxuICAgIGNvbnN0IHJlcXVlc3QgPSBtZXRob2QucmVxdWVzdDtcbiAgICBjb25zdCByZXF1ZXN0VmFyaWFibGVzID0gW107XG4gICAgY29uc3QgaW5wdXRWYXJpYWJsZXMgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICByZXF1ZXN0VmFyaWFibGVzLnB1c2goXG4gICAgICAgIGAkJHtwcm9wLm5hbWV9OiAke3RoaXMuZ3JhcGhxbFR5cGVOYW1lKHByb3AsIHRydWUpfWAsIC8vIHdpdGhvdXQgY2FtZWxDYXNlXG4gICAgICAgIC8vIGAkJHtjYW1lbENhc2UocHJvcC5uYW1lKX06ICR7dGhpcy5ncmFwaHFsVHlwZU5hbWUocHJvcCwgdHJ1ZSl9YCwgLy8gd2l0aCBjYW1lbENhc2VcbiAgICAgICk7XG4gICAgICBpbnB1dFZhcmlhYmxlcy5wdXNoKGAke3Byb3AubmFtZX06ICQke3Byb3AubmFtZX1gKTsgLy8gd2l0aG91dCBjYW1lbENhc2VcbiAgICAgIC8vIGlucHV0VmFyaWFibGVzLnB1c2goYCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9OiAkJHtjYW1lbENhc2UocHJvcC5uYW1lKX1gKTsgLy8gd2l0aCBjYW1lbENhc2VcbiAgICB9XG5cbiAgICBjb25zdCByZXBseSA9IG1ldGhvZC5yZXBseTtcbiAgICBsZXQgZmllbGRMaXN0OiBzdHJpbmc7XG4gICAgaWYgKGFyZ3Mub3ZlcnJpZGVGaWVsZHMpIHtcbiAgICAgIGZpZWxkTGlzdCA9IGAke2FyZ3Mub3ZlcnJpZGVGaWVsZHN9YDtcbiAgICB9IGVsc2Uge1xuICAgICAgZmllbGRMaXN0ID0gdGhpcy5maWVsZExpc3QocmVwbHksIGFyZ3MuYXNzb2NpYXRpb25zLCB0aGlzLnN5c3RlbU9iamVjdCk7XG4gICAgfVxuXG4gICAgY29uc3QgcmVzdWx0U3RyaW5nID0gYHF1ZXJ5ICR7bWV0aG9kTmFtZX0oJHtyZXF1ZXN0VmFyaWFibGVzLmpvaW4oXG4gICAgICBcIiwgXCIsXG4gICAgKX0pIHsgJHttZXRob2ROYW1lfShpbnB1dDogeyAke2lucHV0VmFyaWFibGVzLmpvaW4oXG4gICAgICBcIiwgXCIsXG4gICAgKX0gfSkgeyAke2ZpZWxkTGlzdH0gfSB9YDtcblxuICAgIC8vIExvZyBxdWVyeVxuICAgIC8vIGNvbnNvbGUubG9nKGBxdWVyeSAke3Jlc3VsdFN0cmluZ31gKTtcblxuICAgIHJldHVybiBncWxgXG4gICAgICAke3Jlc3VsdFN0cmluZ31cbiAgICBgO1xuICB9XG5cbiAgbXV0YXRpb24oYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLCAvLyB3aXRob3V0IGNhbWVsQ2FzZVxuICAgICAgICAvLyBgJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9OiAke3RoaXMuZ3JhcGhxbFR5cGVOYW1lKHByb3AsIHRydWUpfWAsIC8vIHdpdGggY2FtZWxDYXNlXG4gICAgICApO1xuICAgICAgaW5wdXRWYXJpYWJsZXMucHVzaChgJHtwcm9wLm5hbWV9OiAkJHtwcm9wLm5hbWV9YCk7IC8vIHdpdGhvdXQgY2FtZWxDYXNlXG4gICAgICAvLyBpbnB1dFZhcmlhYmxlcy5wdXNoKGAke2NhbWVsQ2FzZShwcm9wLm5hbWUpfTogJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9YCk7IC8vIHdpdGggY2FtZWxDYXNlXG4gICAgfVxuXG4gICAgY29uc3QgcmVwbHkgPSBtZXRob2QucmVwbHk7XG4gICAgbGV0IGZpZWxkTGlzdDogc3RyaW5nO1xuICAgIGlmIChhcmdzLm92ZXJyaWRlRmllbGRzKSB7XG4gICAgICBmaWVsZExpc3QgPSBgJHthcmdzLm92ZXJyaWRlRmllbGRzfWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGZpZWxkTGlzdCA9IHRoaXMuZmllbGRMaXN0KHJlcGx5LCBhcmdzLmFzc29jaWF0aW9ucywgdGhpcy5zeXN0ZW1PYmplY3QpO1xuICAgIH1cblxuICAgIGNvbnN0IHJlc3VsdFN0cmluZyA9IGBtdXRhdGlvbiAke21ldGhvZE5hbWV9KCR7cmVxdWVzdFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9KSB7ICR7bWV0aG9kTmFtZX0oaW5wdXQ6IHsgJHtpbnB1dFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9IH0pIHsgJHtmaWVsZExpc3R9IH0gfWA7XG4gICAgY29uc29sZS5sb2cocmVzdWx0U3RyaW5nKTtcbiAgICByZXR1cm4gZ3FsYFxuICAgICAgJHtyZXN1bHRTdHJpbmd9XG4gICAgYDtcbiAgfVxufVxuIl19