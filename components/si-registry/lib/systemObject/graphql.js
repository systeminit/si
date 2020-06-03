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

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

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

      var resultString = "query ".concat(methodName, "(").concat(requestVariables.join(", "), ") { ").concat(methodName, "(input: { ").concat(inputVariables.join(", "), " }) { ").concat(fieldList, " } }");
      console.log("query ".concat(resultString));
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJ2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eSIsInByb3AiLCJyZXBlYXRlZCIsImtpbmQiLCJwcm9wTGluayIsImxvb2t1cE15c2VsZiIsInByb3BPYmplY3QiLCJyZXN1bHQiLCJwcm9wZXJ0aWVzIiwiYXR0cnMiLCJmaWVsZCIsImZpZWxkVmFyaWFibGVzIiwibmFtZSIsIlNpR3JhcGhxbCIsInN5c3RlbU9iamVjdCIsImFyZ3MiLCJtZXRob2QiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJtZXRob2ROYW1lIiwicmVwbHkiLCJsb29rdXBOYW1lIiwib3ZlcnJpZGVOYW1lIiwidHlwZU5hbWUiLCJkYXRhIiwicmVxdWlyZWQiLCJ1bmRlZmluZWQiLCJyZXF1ZXN0IiwiaW5wdXRUeXBlIiwicGFyZW50TmFtZSIsIm51bWJlcktpbmQiLCJsaW5rUHJvcCIsInJlYWxQcm9wIiwiZ3JhcGhxbFR5cGVOYW1lIiwiYXNzb2NpYXRpb25zIiwiYXNzb2NpYXRpb25MaXN0IiwicHVzaCIsImZpZWxkTmFtZSIsImFzc29jT2JqIiwiZ2V0QnlGaWVsZE5hbWUiLCJhc3NvY1N5c3RlbSIsInJlZ2lzdHJ5IiwiZ2V0IiwiYXNzb2NNZXRob2QiLCJmaWVsZExpc3QiLCJqb2luIiwic3lzdGVtT2JqZWN0TWVtbyIsImhpZGRlbiIsInNraXAiLCJhc3NvY2lhdGlvbkZpZWxkTGlzdCIsInJlYWxPYmoiLCJyZXF1ZXN0VmFyaWFibGVzIiwiaW5wdXRWYXJpYWJsZXMiLCJvdmVycmlkZUZpZWxkcyIsInJlc3VsdFN0cmluZyIsImNvbnNvbGUiLCJsb2ciLCJncWwiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUdBOztBQUVBOztBQUNBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBdUJBO0FBQ0E7QUFDTyxTQUFTQSwwQkFBVCxDQUFvQ0MsSUFBcEMsRUFBd0U7QUFBQSxNQUF2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Q0FBQ21CLFlBQTNCLEVBQXlDLEtBQUtwQixZQUE5QyxDQUFaO0FBQ0Q7O0FBRUQsVUFBTXdDLFlBQVksc0JBQWVuQyxVQUFmLGNBQTZCZ0MsZ0JBQWdCLENBQUNOLElBQWpCLENBQzdDLElBRDZDLENBQTdCLGlCQUVWMUIsVUFGVSx1QkFFYWlDLGNBQWMsQ0FBQ1AsSUFBZixDQUM3QixJQUQ2QixDQUZiLG1CQUlSRCxTQUpRLFNBQWxCO0FBS0FXLE1BQUFBLE9BQU8sQ0FBQ0MsR0FBUixDQUFZRixZQUFaO0FBQ0EsaUJBQU9HLHNCQUFQLHNCQUNJSCxZQURKO0FBR0QiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcm9wTWV0aG9kLCBQcm9wT2JqZWN0LCBQcm9wcyB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuaW1wb3J0IHsgUHJvcExpbmsgfSBmcm9tIFwiLi4vcHJvcC9saW5rXCI7XG5pbXBvcnQgeyBPYmplY3RUeXBlcyB9IGZyb20gXCIuLi9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5cbmltcG9ydCB7IHBhc2NhbENhc2UsIGNhbWVsQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuaW1wb3J0IGdxbCBmcm9tIFwiZ3JhcGhxbC10YWdcIjtcbmltcG9ydCB7IERvY3VtZW50Tm9kZSB9IGZyb20gXCJncmFwaHFsXCI7XG5pbXBvcnQgeyBBc3NvY2lhdGlvbiB9IGZyb20gXCIuL2Fzc29jaWF0aW9uc1wiO1xuXG5leHBvcnQgaW50ZXJmYWNlIFF1ZXJ5QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgb3ZlcnJpZGVOYW1lPzogc3RyaW5nO1xuICBvdmVycmlkZUZpZWxkcz86IHN0cmluZztcbiAgYXNzb2NpYXRpb25zPzoge1xuICAgIFtrZXk6IHN0cmluZ106IHN0cmluZ1tdO1xuICB9O1xufVxuXG5leHBvcnQgaW50ZXJmYWNlIFZhcmlhYmxlc09iamVjdEFyZ3Mge1xuICBtZXRob2ROYW1lOiBzdHJpbmc7XG59XG5cbmV4cG9ydCBpbnRlcmZhY2UgVmFsaWRhdGVSZXN1bHRBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xuICBkYXRhOiBSZWNvcmQ8c3RyaW5nLCBhbnk+O1xuICBvdmVycmlkZU5hbWU/OiBzdHJpbmc7XG59XG5cbi8vIFNlY29uZCBhcmd1bWVudCBpcyBpZiB5b3Ugd2FudCBhIHJlcGVhdGVkIGZpZWxkXG4vLyBBS0EgdGhlUG9vcmx5TmFtZWRGdW5jdGlvbigpIDopXG5leHBvcnQgZnVuY3Rpb24gdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkocHJvcDogUHJvcHMsIHJlcGVhdGVkID0gZmFsc2UpOiBhbnkge1xuICBpZiAoXG4gICAgcHJvcC5raW5kKCkgPT0gXCJ0ZXh0XCIgfHxcbiAgICBwcm9wLmtpbmQoKSA9PSBcIm51bWJlclwiIHx8XG4gICAgcHJvcC5raW5kKCkgPT0gXCJjb2RlXCIgfHxcbiAgICBwcm9wLmtpbmQoKSA9PSBcImVudW1cIlxuICApIHtcbiAgICBpZiAocHJvcC5yZXBlYXRlZCAmJiByZXBlYXRlZCkge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJcIjtcbiAgICB9XG4gIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJtYXBcIikge1xuICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9XG4gIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICBjb25zdCBwcm9wTGluayA9IHByb3AgYXMgUHJvcExpbms7XG4gICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9IGVsc2Uge1xuICAgICAgLy8gVE9ETzogVGhlcmUgbWlnaHQgYmUgYSBidWcgaGVyZSwgd2hlcmUgdGhlIG5hbWUgb2YgdGhlIHByb3AgaXRzZWxmXG4gICAgICAvLyBhbmQgdGhlIG5hbWUgb2YgdGhlIGxpbmtlZCBwcm9wIGRvbid0IG1hdGNoLCBhbmQgc28gd2UgZ2V0IHRoZVxuICAgICAgLy8gd3JvbmcgZmllbGQgbmFtZSBpZiB0aGUgcHJvcCBpcyBhbiBvYmplY3QuXG4gICAgICByZXR1cm4gdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkocHJvcExpbmsubG9va3VwTXlzZWxmKCksIHJlcGVhdGVkKTtcbiAgICB9XG4gIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJvYmplY3RcIiB8fCBwcm9wLmtpbmQoKSA9PSBcIm1ldGhvZFwiKSB7XG4gICAgY29uc3QgcHJvcE9iamVjdCA9IHByb3AgYXMgUHJvcE9iamVjdDtcbiAgICBjb25zdCByZXN1bHQ6IFJlY29yZDxzdHJpbmcsIHVua25vd24+ID0ge307XG4gICAgZm9yIChjb25zdCBmaWVsZCBvZiBwcm9wT2JqZWN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGNvbnN0IGZpZWxkVmFyaWFibGVzID0gdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkoZmllbGQsIHJlcGVhdGVkKTtcbiAgICAgIHJlc3VsdFtgJHtmaWVsZC5uYW1lfWBdID0gZmllbGRWYXJpYWJsZXM7XG4gICAgfVxuICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiByZXN1bHQ7XG4gICAgfVxuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBTaUdyYXBocWwge1xuICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogU2lHcmFwaHFsW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICB2YWxpZGF0ZVJlc3VsdChhcmdzOiBWYWxpZGF0ZVJlc3VsdEFyZ3MpOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCByZXBseSA9IG1ldGhvZC5yZXBseTtcbiAgICBjb25zdCBsb29rdXBOYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG4gICAgY29uc3QgcmVzdWx0ID0gYXJncy5kYXRhLmRhdGFbbG9va3VwTmFtZV07XG4gICAgZm9yIChjb25zdCBmaWVsZCBvZiByZXBseS5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAoZmllbGQucmVxdWlyZWQgJiYgcmVzdWx0W2ZpZWxkLm5hbWVdID09IHVuZGVmaW5lZCkge1xuICAgICAgICB0aHJvdyBgcmVzcG9uc2UgaW5jb21wbGV0ZTsgbWlzc2luZyByZXF1aXJlZCBmaWVsZCAke2ZpZWxkfWA7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQ7XG4gIH1cblxuICB2YXJpYWJsZXNPYmplY3QoYXJnczogVmFyaWFibGVzT2JqZWN0QXJncyk6IFJlY29yZDxzdHJpbmcsIGFueT4ge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IHJlcXVlc3QgPSBtZXRob2QucmVxdWVzdDtcbiAgICByZXR1cm4gdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkocmVxdWVzdCwgdHJ1ZSk7XG4gIH1cblxuICBncmFwaHFsVHlwZU5hbWUocHJvcDogUHJvcHMsIGlucHV0VHlwZT86IGJvb2xlYW4pOiBzdHJpbmcge1xuICAgIGxldCByZXN1bHQgPSBcIlwiO1xuICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcIm9iamVjdFwiIHx8IHByb3Aua2luZCgpID09IFwiZW51bVwiKSB7XG4gICAgICBsZXQgcmVxdWVzdCA9IFwiXCI7XG4gICAgICBpZiAoaW5wdXRUeXBlICYmIHByb3Aua2luZCgpICE9IFwiZW51bVwiKSB7XG4gICAgICAgIHJlcXVlc3QgPSBcIlJlcXVlc3RcIjtcbiAgICAgIH1cbiAgICAgIHJlc3VsdCA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9JHtyZXF1ZXN0fWA7XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcInRleHRcIiB8fCBwcm9wLmtpbmQoKSA9PSBcInBhc3N3b3JkXCIpIHtcbiAgICAgIGlmIChwcm9wLm5hbWUgPT0gXCJpZFwiKSB7XG4gICAgICAgIHJlc3VsdCA9IFwiSURcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdCA9IFwiU3RyaW5nXCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm51bWJlclwiKSB7XG4gICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG9uJ3Qga25vdyBhYm91dCBudW1iZXJLaW5kIGJlbG93XG4gICAgICBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50MzJcIikge1xuICAgICAgICByZXN1bHQgPSBcIkludFwiO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0ID0gXCJTdHJpbmdcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgICBjb25zdCBsaW5rUHJvcCA9IHByb3AgYXMgUHJvcExpbms7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IGxpbmtQcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgcmV0dXJuIHRoaXMuZ3JhcGhxbFR5cGVOYW1lKHJlYWxQcm9wLCBpbnB1dFR5cGUpO1xuICAgIH1cbiAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuICAgICAgcmV0dXJuIGAke3Jlc3VsdH0hYDtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHJlc3VsdDtcbiAgICB9XG4gIH1cblxuICBhc3NvY2lhdGlvbkZpZWxkTGlzdChcbiAgICBhc3NvY2lhdGlvbnM6IFF1ZXJ5QXJnc1tcImFzc29jaWF0aW9uc1wiXSxcbiAgICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzLFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IGFzc29jaWF0aW9uTGlzdCA9IGFzc29jaWF0aW9ucyAmJiBhc3NvY2lhdGlvbnNbc3lzdGVtT2JqZWN0LnR5cGVOYW1lXTtcblxuICAgIGlmIChhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYXNzb2NpYXRpb25zIHtcIik7XG4gICAgICBmb3IgKGNvbnN0IGZpZWxkTmFtZSBvZiBhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgICAgY29uc3QgYXNzb2NPYmogPSBzeXN0ZW1PYmplY3QuYXNzb2NpYXRpb25zLmdldEJ5RmllbGROYW1lKGZpZWxkTmFtZSk7XG4gICAgICAgIGNvbnN0IGFzc29jU3lzdGVtID0gcmVnaXN0cnkuZ2V0KGFzc29jT2JqLnR5cGVOYW1lKTtcbiAgICAgICAgY29uc3QgYXNzb2NNZXRob2QgPSBhc3NvY1N5c3RlbS5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgICAgIGFzc29jT2JqLm1ldGhvZE5hbWUsXG4gICAgICAgICkgYXMgUHJvcE1ldGhvZDtcblxuICAgICAgICByZXN1bHQucHVzaChgJHtmaWVsZE5hbWV9IHtgKTtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QoYXNzb2NNZXRob2QucmVwbHksIGFzc29jaWF0aW9ucywgYXNzb2NTeXN0ZW0pLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHQucHVzaChgfWApO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goXCJ9XCIpO1xuICAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiIFwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9XG5cbiAgZmllbGRMaXN0KFxuICAgIHByb3BPYmplY3Q6IFByb3BPYmplY3QsXG4gICAgYXNzb2NpYXRpb25zOiBRdWVyeUFyZ3NbXCJhc3NvY2lhdGlvbnNcIl0sXG4gICAgc3lzdGVtT2JqZWN0TWVtbzogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgbGV0IHN5c3RlbU9iamVjdDtcbiAgICBpZiAoc3lzdGVtT2JqZWN0TWVtbykge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0TWVtbztcbiAgICB9IGVsc2Uge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gdGhpcy5zeXN0ZW1PYmplY3Q7XG4gICAgfVxuICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4gfHwgcHJvcC5za2lwKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goYCR7cHJvcC5uYW1lfWApOyAvLyB3aXRob3V0IGNhbWVsQ2FzZVxuICAgICAgLy8gcmVzdWx0LnB1c2goYCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9YCk7IC8vIHdpdGggY2FtZWxDYXNlXG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICByZXN1bHQucHVzaChcIntcIik7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KHByb3AgYXMgUHJvcE9iamVjdCwgdW5kZWZpbmVkLCBzeXN0ZW1PYmplY3QpLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHQucHVzaCh0aGlzLmFzc29jaWF0aW9uRmllbGRMaXN0KGFzc29jaWF0aW9ucywgc3lzdGVtT2JqZWN0KSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcIm1hcFwiKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKFwieyBrZXkgdmFsdWUgfVwiKTtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgICBjb25zdCByZWFsT2JqID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgICAgaWYgKHJlYWxPYmoua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChcIntcIik7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QocmVhbE9iaiBhcyBQcm9wT2JqZWN0LCB1bmRlZmluZWQsIHN5c3RlbU9iamVjdCksXG4gICAgICAgICk7XG4gICAgICAgIGlmIChyZWFsT2JqLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2godGhpcy5hc3NvY2lhdGlvbkZpZWxkTGlzdChhc3NvY2lhdGlvbnMsIHN5c3RlbU9iamVjdCkpO1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gYCR7cmVzdWx0LmpvaW4oXCIgXCIpfWA7XG4gIH1cblxuICBxdWVyeShhcmdzOiBRdWVyeUFyZ3MpOiBEb2N1bWVudE5vZGUge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IG1ldGhvZE5hbWUgPVxuICAgICAgYXJncy5vdmVycmlkZU5hbWUgfHxcbiAgICAgIGAke2NhbWVsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9JHtwYXNjYWxDYXNlKGFyZ3MubWV0aG9kTmFtZSl9YDtcblxuICAgIGNvbnN0IHJlcXVlc3QgPSBtZXRob2QucmVxdWVzdDtcbiAgICBjb25zdCByZXF1ZXN0VmFyaWFibGVzID0gW107XG4gICAgY29uc3QgaW5wdXRWYXJpYWJsZXMgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICByZXF1ZXN0VmFyaWFibGVzLnB1c2goXG4gICAgICAgIGAkJHtwcm9wLm5hbWV9OiAke3RoaXMuZ3JhcGhxbFR5cGVOYW1lKHByb3AsIHRydWUpfWAsIC8vIHdpdGhvdXQgY2FtZWxDYXNlXG4gICAgICAgIC8vIGAkJHtjYW1lbENhc2UocHJvcC5uYW1lKX06ICR7dGhpcy5ncmFwaHFsVHlwZU5hbWUocHJvcCwgdHJ1ZSl9YCwgLy8gd2l0aCBjYW1lbENhc2VcbiAgICAgICk7XG4gICAgICBpbnB1dFZhcmlhYmxlcy5wdXNoKGAke3Byb3AubmFtZX06ICQke3Byb3AubmFtZX1gKTsgLy8gd2l0aG91dCBjYW1lbENhc2VcbiAgICAgIC8vIGlucHV0VmFyaWFibGVzLnB1c2goYCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9OiAkJHtjYW1lbENhc2UocHJvcC5uYW1lKX1gKTsgLy8gd2l0aCBjYW1lbENhc2VcbiAgICB9XG5cbiAgICBjb25zdCByZXBseSA9IG1ldGhvZC5yZXBseTtcbiAgICBsZXQgZmllbGRMaXN0OiBzdHJpbmc7XG4gICAgaWYgKGFyZ3Mub3ZlcnJpZGVGaWVsZHMpIHtcbiAgICAgIGZpZWxkTGlzdCA9IGAke2FyZ3Mub3ZlcnJpZGVGaWVsZHN9YDtcbiAgICB9IGVsc2Uge1xuICAgICAgZmllbGRMaXN0ID0gdGhpcy5maWVsZExpc3QocmVwbHksIGFyZ3MuYXNzb2NpYXRpb25zLCB0aGlzLnN5c3RlbU9iamVjdCk7XG4gICAgfVxuXG4gICAgY29uc3QgcmVzdWx0U3RyaW5nID0gYHF1ZXJ5ICR7bWV0aG9kTmFtZX0oJHtyZXF1ZXN0VmFyaWFibGVzLmpvaW4oXG4gICAgICBcIiwgXCIsXG4gICAgKX0pIHsgJHttZXRob2ROYW1lfShpbnB1dDogeyAke2lucHV0VmFyaWFibGVzLmpvaW4oXG4gICAgICBcIiwgXCIsXG4gICAgKX0gfSkgeyAke2ZpZWxkTGlzdH0gfSB9YDtcbiAgICBjb25zb2xlLmxvZyhgcXVlcnkgJHtyZXN1bHRTdHJpbmd9YCk7XG4gICAgcmV0dXJuIGdxbGBcbiAgICAgICR7cmVzdWx0U3RyaW5nfVxuICAgIGA7XG4gIH1cblxuICBtdXRhdGlvbihhcmdzOiBRdWVyeUFyZ3MpOiBEb2N1bWVudE5vZGUge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IG1ldGhvZE5hbWUgPVxuICAgICAgYXJncy5vdmVycmlkZU5hbWUgfHxcbiAgICAgIGAke2NhbWVsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9JHtwYXNjYWxDYXNlKGFyZ3MubWV0aG9kTmFtZSl9YDtcblxuICAgIGNvbnN0IHJlcXVlc3QgPSBtZXRob2QucmVxdWVzdDtcbiAgICBjb25zdCByZXF1ZXN0VmFyaWFibGVzID0gW107XG4gICAgY29uc3QgaW5wdXRWYXJpYWJsZXMgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICByZXF1ZXN0VmFyaWFibGVzLnB1c2goXG4gICAgICAgIGAkJHtwcm9wLm5hbWV9OiAke3RoaXMuZ3JhcGhxbFR5cGVOYW1lKHByb3AsIHRydWUpfWAsIC8vIHdpdGhvdXQgY2FtZWxDYXNlXG4gICAgICAgIC8vIGAkJHtjYW1lbENhc2UocHJvcC5uYW1lKX06ICR7dGhpcy5ncmFwaHFsVHlwZU5hbWUocHJvcCwgdHJ1ZSl9YCwgLy8gd2l0aCBjYW1lbENhc2VcbiAgICAgICk7XG4gICAgICBpbnB1dFZhcmlhYmxlcy5wdXNoKGAke3Byb3AubmFtZX06ICQke3Byb3AubmFtZX1gKTsgLy8gd2l0aG91dCBjYW1lbENhc2VcbiAgICAgIC8vIGlucHV0VmFyaWFibGVzLnB1c2goYCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9OiAkJHtjYW1lbENhc2UocHJvcC5uYW1lKX1gKTsgLy8gd2l0aCBjYW1lbENhc2VcbiAgICB9XG5cbiAgICBjb25zdCByZXBseSA9IG1ldGhvZC5yZXBseTtcbiAgICBsZXQgZmllbGRMaXN0OiBzdHJpbmc7XG4gICAgaWYgKGFyZ3Mub3ZlcnJpZGVGaWVsZHMpIHtcbiAgICAgIGZpZWxkTGlzdCA9IGAke2FyZ3Mub3ZlcnJpZGVGaWVsZHN9YDtcbiAgICB9IGVsc2Uge1xuICAgICAgZmllbGRMaXN0ID0gdGhpcy5maWVsZExpc3QocmVwbHksIGFyZ3MuYXNzb2NpYXRpb25zLCB0aGlzLnN5c3RlbU9iamVjdCk7XG4gICAgfVxuXG4gICAgY29uc3QgcmVzdWx0U3RyaW5nID0gYG11dGF0aW9uICR7bWV0aG9kTmFtZX0oJHtyZXF1ZXN0VmFyaWFibGVzLmpvaW4oXG4gICAgICBcIiwgXCIsXG4gICAgKX0pIHsgJHttZXRob2ROYW1lfShpbnB1dDogeyAke2lucHV0VmFyaWFibGVzLmpvaW4oXG4gICAgICBcIiwgXCIsXG4gICAgKX0gfSkgeyAke2ZpZWxkTGlzdH0gfSB9YDtcbiAgICBjb25zb2xlLmxvZyhyZXN1bHRTdHJpbmcpO1xuICAgIHJldHVybiBncWxgXG4gICAgICAke3Jlc3VsdFN0cmluZ31cbiAgICBgO1xuICB9XG59XG4iXX0=