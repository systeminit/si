"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.SiGraphql = void 0;

var _taggedTemplateLiteral2 = _interopRequireDefault(require("@babel/runtime/helpers/taggedTemplateLiteral"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _registry = require("../registry");

var _changeCase = require("change-case");

var _graphqlTag = _interopRequireDefault(require("graphql-tag"));

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

      var _iterator = _createForOfIteratorHelper(reply.properties.attrs),
          _step;

      try {
        for (_iterator.s(); !(_step = _iterator.n()).done;) {
          var field = _step.value;

          if (field.required && result[field.name] == undefined) {
            throw "response incomplete; missing required field ".concat(field);
          }
        }
      } catch (err) {
        _iterator.e(err);
      } finally {
        _iterator.f();
      }

      return result;
    }
  }, {
    key: "variablesObject",
    value: function variablesObject(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var request = method.request;
      var result = {};

      var _iterator2 = _createForOfIteratorHelper(request.properties.attrs),
          _step2;

      try {
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          var field = _step2.value;
          result[field.name] = field.defaultValue();
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }

      return result;
    }
  }, {
    key: "graphqlTypeName",
    value: function graphqlTypeName(prop, inputType) {
      var result = "";

      if (prop.kind() == "object" || prop.kind() == "enum") {
        var request = "";

        if (inputType) {
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
        result = "Int";
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

          result.push("".concat(prop.name));

          if (prop.kind() == "object") {
            result.push("{");
            result.push(this.fieldList(prop, undefined, systemObject));
            result.push(this.associationFieldList(associations, systemObject));
            result.push("}");
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
          requestVariables.push("$".concat(prop.name, ": ").concat(this.graphqlTypeName(prop)));
          inputVariables.push("".concat(prop.name, ": $").concat(prop.name));
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
      return (0, _graphqlTag["default"])(_templateObject(), resultString);
    }
  }]);
  return SiGraphql;
}();

exports.SiGraphql = SiGraphql;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJTaUdyYXBocWwiLCJzeXN0ZW1PYmplY3QiLCJhcmdzIiwibWV0aG9kIiwibWV0aG9kcyIsImdldEVudHJ5IiwibWV0aG9kTmFtZSIsInJlcGx5IiwibG9va3VwTmFtZSIsIm92ZXJyaWRlTmFtZSIsInR5cGVOYW1lIiwicmVzdWx0IiwiZGF0YSIsInByb3BlcnRpZXMiLCJhdHRycyIsImZpZWxkIiwicmVxdWlyZWQiLCJuYW1lIiwidW5kZWZpbmVkIiwicmVxdWVzdCIsImRlZmF1bHRWYWx1ZSIsInByb3AiLCJpbnB1dFR5cGUiLCJraW5kIiwicGFyZW50TmFtZSIsImFzc29jaWF0aW9ucyIsImFzc29jaWF0aW9uTGlzdCIsInB1c2giLCJmaWVsZE5hbWUiLCJhc3NvY09iaiIsImdldEJ5RmllbGROYW1lIiwiYXNzb2NTeXN0ZW0iLCJyZWdpc3RyeSIsImdldCIsImFzc29jTWV0aG9kIiwiZmllbGRMaXN0Iiwiam9pbiIsInByb3BPYmplY3QiLCJzeXN0ZW1PYmplY3RNZW1vIiwiaGlkZGVuIiwic2tpcCIsImFzc29jaWF0aW9uRmllbGRMaXN0IiwicmVhbE9iaiIsImxvb2t1cE15c2VsZiIsInJlcXVlc3RWYXJpYWJsZXMiLCJpbnB1dFZhcmlhYmxlcyIsImdyYXBocWxUeXBlTmFtZSIsIm92ZXJyaWRlRmllbGRzIiwicmVzdWx0U3RyaW5nIiwiZ3FsIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7OztBQUVBOztBQUVBOztBQUNBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7SUF1QmFBLFM7QUFHWCxxQkFBWUMsWUFBWixFQUFxRDtBQUFBO0FBQUE7QUFDbkQsU0FBS0EsWUFBTCxHQUFvQkEsWUFBcEI7QUFDRDs7OzttQ0FFY0MsSSxFQUErQztBQUM1RCxVQUFNQyxNQUFNLEdBQUcsS0FBS0YsWUFBTCxDQUFrQkcsT0FBbEIsQ0FBMEJDLFFBQTFCLENBQ2JILElBQUksQ0FBQ0ks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RCxVQUFNK0MsWUFBWSxtQkFBWTFDLFVBQVosY0FBMEJzQyxnQkFBZ0IsQ0FBQ1IsSUFBakIsQ0FDMUMsSUFEMEMsQ0FBMUIsaUJBRVY5QixVQUZVLHVCQUVhdUMsY0FBYyxDQUFDVCxJQUFmLENBQzdCLElBRDZCLENBRmIsbUJBSVJELFNBSlEsU0FBbEI7QUFLQSxpQkFBT2Msc0JBQVAscUJBQ0lELFlBREo7QUFHRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3BNZXRob2QsIFByb3BPYmplY3QsIFByb3BzIH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5pbXBvcnQgeyBPYmplY3RUeXBlcyB9IGZyb20gXCIuLi9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5cbmltcG9ydCB7IHBhc2NhbENhc2UsIGNhbWVsQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuaW1wb3J0IGdxbCBmcm9tIFwiZ3JhcGhxbC10YWdcIjtcbmltcG9ydCB7IERvY3VtZW50Tm9kZSB9IGZyb20gXCJncmFwaHFsXCI7XG5pbXBvcnQgeyBBc3NvY2lhdGlvbiB9IGZyb20gXCIuL2Fzc29jaWF0aW9uc1wiO1xuXG5pbnRlcmZhY2UgUXVlcnlBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xuICBvdmVycmlkZU5hbWU/OiBzdHJpbmc7XG4gIG92ZXJyaWRlRmllbGRzPzogc3RyaW5nO1xuICBhc3NvY2lhdGlvbnM/OiB7XG4gICAgW2tleTogc3RyaW5nXTogc3RyaW5nW107XG4gIH07XG59XG5cbmludGVyZmFjZSBWYXJpYWJsZXNPYmplY3RBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xufVxuXG5pbnRlcmZhY2UgVmFsaWRhdGVSZXN1bHRBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xuICBkYXRhOiBSZWNvcmQ8c3RyaW5nLCBhbnk+O1xuICBvdmVycmlkZU5hbWU/OiBzdHJpbmc7XG59XG5cbmV4cG9ydCBjbGFzcyBTaUdyYXBocWwge1xuICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogU2lHcmFwaHFsW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICB2YWxpZGF0ZVJlc3VsdChhcmdzOiBWYWxpZGF0ZVJlc3VsdEFyZ3MpOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCByZXBseSA9IG1ldGhvZC5yZXBseTtcbiAgICBjb25zdCBsb29rdXBOYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG4gICAgY29uc3QgcmVzdWx0ID0gYXJncy5kYXRhLmRhdGFbbG9va3VwTmFtZV07XG4gICAgZm9yIChjb25zdCBmaWVsZCBvZiByZXBseS5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAoZmllbGQucmVxdWlyZWQgJiYgcmVzdWx0W2ZpZWxkLm5hbWVdID09IHVuZGVmaW5lZCkge1xuICAgICAgICB0aHJvdyBgcmVzcG9uc2UgaW5jb21wbGV0ZTsgbWlzc2luZyByZXF1aXJlZCBmaWVsZCAke2ZpZWxkfWA7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQ7XG4gIH1cblxuICB2YXJpYWJsZXNPYmplY3QoYXJnczogVmFyaWFibGVzT2JqZWN0QXJncyk6IFJlY29yZDxzdHJpbmcsIGFueT4ge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IHJlcXVlc3QgPSBtZXRob2QucmVxdWVzdDtcbiAgICBjb25zdCByZXN1bHQ6IFJlY29yZDxzdHJpbmcsIGFueT4gPSB7fTtcbiAgICBmb3IgKGNvbnN0IGZpZWxkIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVzdWx0W2ZpZWxkLm5hbWVdID0gZmllbGQuZGVmYXVsdFZhbHVlKCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQ7XG4gIH1cblxuICBncmFwaHFsVHlwZU5hbWUocHJvcDogUHJvcHMsIGlucHV0VHlwZT86IGJvb2xlYW4pOiBzdHJpbmcge1xuICAgIGxldCByZXN1bHQgPSBcIlwiO1xuICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcIm9iamVjdFwiIHx8IHByb3Aua2luZCgpID09IFwiZW51bVwiKSB7XG4gICAgICBsZXQgcmVxdWVzdCA9IFwiXCI7XG4gICAgICBpZiAoaW5wdXRUeXBlKSB7XG4gICAgICAgIHJlcXVlc3QgPSBcIlJlcXVlc3RcIjtcbiAgICAgIH1cbiAgICAgIHJlc3VsdCA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9JHtyZXF1ZXN0fWA7XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcInRleHRcIiB8fCBwcm9wLmtpbmQoKSA9PSBcInBhc3N3b3JkXCIpIHtcbiAgICAgIGlmIChwcm9wLm5hbWUgPT0gXCJpZFwiKSB7XG4gICAgICAgIHJlc3VsdCA9IFwiSURcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdCA9IFwiU3RyaW5nXCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm51bWJlclwiKSB7XG4gICAgICByZXN1bHQgPSBcIkludFwiO1xuICAgIH1cblxuICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4gICAgICByZXR1cm4gYCR7cmVzdWx0fSFgO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gcmVzdWx0O1xuICAgIH1cbiAgfVxuXG4gIGFzc29jaWF0aW9uRmllbGRMaXN0KFxuICAgIGFzc29jaWF0aW9uczogUXVlcnlBcmdzW1wiYXNzb2NpYXRpb25zXCJdLFxuICAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgYXNzb2NpYXRpb25MaXN0ID0gYXNzb2NpYXRpb25zICYmIGFzc29jaWF0aW9uc1tzeXN0ZW1PYmplY3QudHlwZU5hbWVdO1xuICAgIGlmIChhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYXNzb2NpYXRpb25zIHtcIik7XG4gICAgICBmb3IgKGNvbnN0IGZpZWxkTmFtZSBvZiBhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgICAgY29uc3QgYXNzb2NPYmogPSBzeXN0ZW1PYmplY3QuYXNzb2NpYXRpb25zLmdldEJ5RmllbGROYW1lKGZpZWxkTmFtZSk7XG4gICAgICAgIGNvbnN0IGFzc29jU3lzdGVtID0gcmVnaXN0cnkuZ2V0KGFzc29jT2JqLnR5cGVOYW1lKTtcbiAgICAgICAgY29uc3QgYXNzb2NNZXRob2QgPSBhc3NvY1N5c3RlbS5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgICAgIGFzc29jT2JqLm1ldGhvZE5hbWUsXG4gICAgICAgICkgYXMgUHJvcE1ldGhvZDtcblxuICAgICAgICByZXN1bHQucHVzaChgJHtmaWVsZE5hbWV9IHtgKTtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QoYXNzb2NNZXRob2QucmVwbHksIGFzc29jaWF0aW9ucywgYXNzb2NTeXN0ZW0pLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHQucHVzaChgfWApO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goXCJ9XCIpO1xuICAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiIFwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9XG5cbiAgZmllbGRMaXN0KFxuICAgIHByb3BPYmplY3Q6IFByb3BPYmplY3QsXG4gICAgYXNzb2NpYXRpb25zOiBRdWVyeUFyZ3NbXCJhc3NvY2lhdGlvbnNcIl0sXG4gICAgc3lzdGVtT2JqZWN0TWVtbzogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgbGV0IHN5c3RlbU9iamVjdDtcbiAgICBpZiAoc3lzdGVtT2JqZWN0TWVtbykge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0TWVtbztcbiAgICB9IGVsc2Uge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gdGhpcy5zeXN0ZW1PYmplY3Q7XG4gICAgfVxuICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4gfHwgcHJvcC5za2lwKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goYCR7cHJvcC5uYW1lfWApO1xuICAgICAgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICB0aGlzLmZpZWxkTGlzdChwcm9wIGFzIFByb3BPYmplY3QsIHVuZGVmaW5lZCwgc3lzdGVtT2JqZWN0KSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0LnB1c2godGhpcy5hc3NvY2lhdGlvbkZpZWxkTGlzdChhc3NvY2lhdGlvbnMsIHN5c3RlbU9iamVjdCkpO1xuICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgY29uc3QgcmVhbE9iaiA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICAgIGlmIChyZWFsT2JqLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICB9XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KHJlYWxPYmogYXMgUHJvcE9iamVjdCwgdW5kZWZpbmVkLCBzeXN0ZW1PYmplY3QpLFxuICAgICAgICApO1xuICAgICAgICBpZiAocmVhbE9iai5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKHRoaXMuYXNzb2NpYXRpb25GaWVsZExpc3QoYXNzb2NpYXRpb25zLCBzeXN0ZW1PYmplY3QpKTtcbiAgICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIGAke3Jlc3VsdC5qb2luKFwiIFwiKX1gO1xuICB9XG5cbiAgcXVlcnkoYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKGAkJHtwcm9wLm5hbWV9OiAke3RoaXMuZ3JhcGhxbFR5cGVOYW1lKHByb3ApfWApO1xuICAgICAgaW5wdXRWYXJpYWJsZXMucHVzaChgJHtwcm9wLm5hbWV9OiAkJHtwcm9wLm5hbWV9YCk7XG4gICAgfVxuXG4gICAgY29uc3QgcmVwbHkgPSBtZXRob2QucmVwbHk7XG4gICAgbGV0IGZpZWxkTGlzdDogc3RyaW5nO1xuICAgIGlmIChhcmdzLm92ZXJyaWRlRmllbGRzKSB7XG4gICAgICBmaWVsZExpc3QgPSBgJHthcmdzLm92ZXJyaWRlRmllbGRzfWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGZpZWxkTGlzdCA9IHRoaXMuZmllbGRMaXN0KHJlcGx5LCBhcmdzLmFzc29jaWF0aW9ucywgdGhpcy5zeXN0ZW1PYmplY3QpO1xuICAgIH1cblxuICAgIGNvbnN0IHJlc3VsdFN0cmluZyA9IGBxdWVyeSAke21ldGhvZE5hbWV9KCR7cmVxdWVzdFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9KSB7ICR7bWV0aG9kTmFtZX0oaW5wdXQ6IHsgJHtpbnB1dFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9IH0pIHsgJHtmaWVsZExpc3R9IH0gfWA7XG4gICAgcmV0dXJuIGdxbGBcbiAgICAgICR7cmVzdWx0U3RyaW5nfVxuICAgIGA7XG4gIH1cbn1cbiJdfQ==