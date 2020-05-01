"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.registry = exports.Registry = void 0;

var _systemComponent = require("./systemComponent");

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

var Registry = /*#__PURE__*/function () {
  function Registry() {
    _classCallCheck(this, Registry);

    _defineProperty(this, "objects", void 0);

    this.objects = [];
  }

  _createClass(Registry, [{
    key: "get",
    value: function get(typeName) {
      var result = this.objects.find(function (v) {
        return v.typeName == typeName;
      });

      if (result) {
        return result;
      } else {
        throw "Cannot get object named ".concat(typeName, " in the registry");
      }
    }
  }, {
    key: "serviceNames",
    value: function serviceNames() {
      var names = new Set();

      var _iterator = _createForOfIteratorHelper(this.objects),
          _step;

      try {
        for (_iterator.s(); !(_step = _iterator.n()).done;) {
          var object = _step.value;

          if (object.serviceName) {
            names.add(object.serviceName);
          }
        }
      } catch (err) {
        _iterator.e(err);
      } finally {
        _iterator.f();
      }

      var arrayNames = [];

      var _iterator2 = _createForOfIteratorHelper(names.values()),
          _step2;

      try {
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          var name = _step2.value;
          arrayNames.push(name);
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }

      return arrayNames;
    }
  }, {
    key: "getObjectsForServiceName",
    value: function getObjectsForServiceName(serviceName) {
      var results = [];

      var _iterator3 = _createForOfIteratorHelper(this.objects),
          _step3;

      try {
        for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
          var object = _step3.value;

          if (object.serviceName == serviceName) {
            results.push(object);
          }
        }
      } catch (err) {
        _iterator3.e(err);
      } finally {
        _iterator3.f();
      }

      return results;
    } // Find a property!

  }, {
    key: "lookupProp",
    value: function lookupProp(lookup) {
      var foundObject = this.objects.find(function (c) {
        return c.typeName == lookup.typeName;
      });

      if (!foundObject) {
        throw "Cannot find object: ".concat(foundObject);
      }

      if (!lookup.names) {
        return foundObject.rootProp;
      }

      var firstName = lookup.names[0];
      var returnProp = foundObject.fields.getEntry(firstName);

      if (!returnProp) {
        throw "Cannot find prop on object ".concat(foundObject.typeName, ": ").concat(firstName);
      }

      if (returnProp.kind() != "object" && lookup.names.length > 1) {
        throw "You asked for sub-properties of a non-object type on ".concat(foundObject.typeName, " property ").concat(firstName);
      }

      for (var i = 1; i < lookup.names.length; i++) {
        var lookupName = lookup.names[i];
        var lookupResult = returnProp["properties"].getEntry(lookupName);

        if (!lookupResult) {
          throw "Cannot find prop \"".concat(lookupName, "\" on ").concat(returnProp.name);
        }

        if (i != lookup.names.length - 1 && lookupResult.kind() != "object") {
          console.log({
            i: i,
            length: lookup.names.length,
            lookupName: lookupName,
            lookupResult: lookupResult
          });
          throw "Cannot look up a sub-property of a non object Prop: ".concat(foundObject.typeName, " property ").concat(lookupName, " is ").concat(lookupResult.kind());
        }

        returnProp = lookupResult;
      }

      return returnProp;
    } // These are "basic" objects - they don't have any extra behavior or
    // automatic fields. They just store the fields you give them.

  }, {
    key: "base",
    value: function base(constructorArgs) {
      var compy = new _systemComponent.BaseObject(constructorArgs);
      this.objects.push(compy);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    } // These are "system" objects - they have what is needed to be an object
    // inside our system. They come with things like types, IDs, tenancy,
    // etc.

  }, {
    key: "system",
    value: function system(constructorArgs) {
      var compy = new _systemComponent.SystemObject(constructorArgs);
      this.objects.push(compy);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    }
  }, {
    key: "component",
    value: function component(constructorArgs) {
      var compy = new _systemComponent.ComponentObject(constructorArgs);
      this.objects.push(compy);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    }
  }, {
    key: "entity",
    value: function entity(constructorArgs) {
      var compy = new _systemComponent.EntityObject(constructorArgs);
      this.objects.push(compy);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    }
  }, {
    key: "componentAndEntity",
    value: function componentAndEntity(constructorArgs) {
      var compy = new _systemComponent.ComponentAndEntityObject(constructorArgs);
      this.objects.push(compy.component);
      this.objects.push(compy.entity);
      this.objects.push(compy.entityEvent);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    }
  }]);

  return Registry;
}();

exports.Registry = Registry;
var registry = new Registry();
exports.registry = registry;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9yZWdpc3RyeS50cyJdLCJuYW1lcyI6WyJSZWdpc3RyeSIsIm9iamVjdHMiLCJ0eXBlTmFtZSIsInJlc3VsdCIsImZpbmQiLCJ2IiwibmFtZXMiLCJTZXQiLCJvYmplY3QiLCJzZXJ2aWNlTmFtZSIsImFkZCIsImFycmF5TmFtZXMiLCJ2YWx1ZXMiLCJuYW1lIiwicHVzaCIsInJlc3VsdHMiLCJsb29rdXAiLCJmb3VuZE9iamVjdCIsImMiLCJyb290UHJvcCIsImZpcnN0TmFtZSIsInJldHVyblByb3AiLCJmaWVsZHMiLCJnZXRFbnRyeSIsImtpbmQiLCJsZW5ndGgiLCJpIiwibG9va3VwTmFtZSIsImxvb2t1cFJlc3VsdCIsImNvbnNvbGUiLCJsb2ciLCJjb25zdHJ1Y3RvckFyZ3MiLCJjb21weSIsIkJhc2VPYmplY3QiLCJvcHRpb25zIiwiU3lzdGVtT2JqZWN0IiwiQ29tcG9uZW50T2JqZWN0IiwiRW50aXR5T2JqZWN0IiwiQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IiwiY29tcG9uZW50IiwiZW50aXR5IiwiZW50aXR5RXZlbnQiLCJyZWdpc3RyeSJdLCJtYXBwaW5ncyI6Ijs7Ozs7OztBQUFBOzs7Ozs7Ozs7Ozs7Ozs7O0lBaUJhQSxRO0FBR1gsc0JBQWM7QUFBQTs7QUFBQTs7QUFDWixTQUFLQyxPQUFMLEdBQWUsRUFBZjtBQUNEOzs7O3dCQUVHQyxRLEVBQTJDO0FBQzdDLFVBQU1DLE1BQU0sR0FBRyxLQUFLRixPQUFMLENBQWFHLElBQWIsQ0FBa0IsVUFBQUMsQ0FBQztBQUFBLGVBQUlBLENBQUMsQ0FBQ0gsUUFBRixJQUFjQSxRQUFsQjtBQUFBLE9BQW5CLENBQWY7O0FBQ0EsVUFBSUMsTUFBSixFQUFZO0FBQ1YsZUFBT0EsTUFBUDtBQUNELE9BRkQsTUFFTztBQUNMLGdEQUFpQ0QsUUFBakM7QUFDRDtBQUNGOzs7bUNBRXdCO0FBQ3ZCLFVBQU1JLEtBQUssR0FBRyxJQUFJQyxHQUFKLEVBQWQ7O0FBRHVCLGlEQUVGLEtBQUtOLE9B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Syw2QkFBSixDQUFpQk4sZUFBakIsQ0FBZDtBQUNBLFdBQUs5QixPQUFMLENBQWFhLElBQWIsQ0FBa0JrQixLQUFsQjs7QUFDQSxVQUFJRCxlQUFlLENBQUNHLE9BQXBCLEVBQTZCO0FBQzNCSCxRQUFBQSxlQUFlLENBQUNHLE9BQWhCLENBQXdCRixLQUF4QjtBQUNEOztBQUNELGFBQU9BLEtBQVA7QUFDRDs7O3VDQUdDRCxlLEVBQzBCO0FBQzFCLFVBQU1DLEtBQUssR0FBRyxJQUFJTSx5Q0FBSixDQUE2QlAsZUFBN0IsQ0FBZDtBQUNBLFdBQUs5QixPQUFMLENBQWFhLElBQWIsQ0FBa0JrQixLQUFLLENBQUNPLFNBQXhCO0FBQ0EsV0FBS3RDLE9BQUwsQ0FBYWEsSUFBYixDQUFrQmtCLEtBQUssQ0FBQ1EsTUFBeEI7QUFDQSxXQUFLdkMsT0FBTCxDQUFhYSxJQUFiLENBQWtCa0IsS0FBSyxDQUFDUyxXQUF4Qjs7QUFDQSxVQUFJVixlQUFlLENBQUNHLE9BQXBCLEVBQTZCO0FBQzNCSCxRQUFBQSxlQUFlLENBQUNHLE9BQWhCLENBQXdCRixLQUF4QjtBQUNEOztBQUNELGFBQU9BLEtBQVA7QUFDRDs7Ozs7OztBQUdJLElBQU1VLFFBQVEsR0FBRyxJQUFJMUMsUUFBSixFQUFqQiIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7XG4gIE9iamVjdFR5cGVzLFxuICBCYXNlT2JqZWN0Q29uc3RydWN0b3IsXG4gIFN5c3RlbU9iamVjdCxcbiAgQmFzZU9iamVjdCxcbiAgQ29tcG9uZW50T2JqZWN0LFxuICBFbnRpdHlPYmplY3QsXG4gIENvbXBvbmVudEFuZEVudGl0eU9iamVjdCxcbiAgQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0Q29uc3RydWN0b3IsXG59IGZyb20gXCIuL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IHsgUHJvcHMgfSBmcm9tIFwiLi9hdHRyTGlzdFwiO1xuXG5leHBvcnQgaW50ZXJmYWNlIFByb3BMb29rdXAge1xuICB0eXBlTmFtZTogc3RyaW5nO1xuICBuYW1lcz86IHN0cmluZ1tdO1xufVxuXG5leHBvcnQgY2xhc3MgUmVnaXN0cnkge1xuICBvYmplY3RzOiBPYmplY3RUeXBlc1tdO1xuXG4gIGNvbnN0cnVjdG9yKCkge1xuICAgIHRoaXMub2JqZWN0cyA9IFtdO1xuICB9XG5cbiAgZ2V0KHR5cGVOYW1lOiBzdHJpbmcpOiBPYmplY3RUeXBlcyB8IHVuZGVmaW5lZCB7XG4gICAgY29uc3QgcmVzdWx0ID0gdGhpcy5vYmplY3RzLmZpbmQodiA9PiB2LnR5cGVOYW1lID09IHR5cGVOYW1lKTtcbiAgICBpZiAocmVzdWx0KSB7XG4gICAgICByZXR1cm4gcmVzdWx0O1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBgQ2Fubm90IGdldCBvYmplY3QgbmFtZWQgJHt0eXBlTmFtZX0gaW4gdGhlIHJlZ2lzdHJ5YDtcbiAgICB9XG4gIH1cblxuICBzZXJ2aWNlTmFtZXMoKTogc3RyaW5nW10ge1xuICAgIGNvbnN0IG5hbWVzID0gbmV3IFNldCgpO1xuICAgIGZvciAoY29uc3Qgb2JqZWN0IG9mIHRoaXMub2JqZWN0cykge1xuICAgICAgaWYgKG9iamVjdC5zZXJ2aWNlTmFtZSkge1xuICAgICAgICBuYW1lcy5hZGQob2JqZWN0LnNlcnZpY2VOYW1lKTtcbiAgICAgIH1cbiAgICB9XG4gICAgY29uc3QgYXJyYXlOYW1lcyA9IFtdO1xuICAgIGZvciAoY29uc3QgbmFtZSBvZiBuYW1lcy52YWx1ZXMoKSkge1xuICAgICAgYXJyYXlOYW1lcy5wdXNoKG5hbWUpO1xuICAgIH1cbiAgICByZXR1cm4gYXJyYXlOYW1lcztcbiAgfVxuXG4gIGdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZTogc3RyaW5nKTogT2JqZWN0VHlwZXNbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGZvciAoY29uc3Qgb2JqZWN0IG9mIHRoaXMub2JqZWN0cykge1xuICAgICAgaWYgKG9iamVjdC5zZXJ2aWNlTmFtZSA9PSBzZXJ2aWNlTmFtZSkge1xuICAgICAgICByZXN1bHRzLnB1c2gob2JqZWN0KTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHM7XG4gIH1cblxuICAvLyBGaW5kIGEgcHJvcGVydHkhXG4gIGxvb2t1cFByb3AobG9va3VwOiBQcm9wTG9va3VwKTogUHJvcHMge1xuICAgIGNvbnN0IGZvdW5kT2JqZWN0ID0gdGhpcy5vYmplY3RzLmZpbmQoYyA9PiBjLnR5cGVOYW1lID09IGxvb2t1cC50eXBlTmFtZSk7XG4gICAgaWYgKCFmb3VuZE9iamVjdCkge1xuICAgICAgdGhyb3cgYENhbm5vdCBmaW5kIG9iamVjdDogJHtmb3VuZE9iamVjdH1gO1xuICAgIH1cbiAgICBpZiAoIWxvb2t1cC5uYW1lcykge1xuICAgICAgcmV0dXJuIGZvdW5kT2JqZWN0LnJvb3RQcm9wO1xuICAgIH1cbiAgICBjb25zdCBmaXJzdE5hbWUgPSBsb29rdXAubmFtZXNbMF07XG4gICAgbGV0IHJldHVyblByb3AgPSBmb3VuZE9iamVjdC5maWVsZHMuZ2V0RW50cnkoZmlyc3ROYW1lKTtcbiAgICBpZiAoIXJldHVyblByb3ApIHtcbiAgICAgIHRocm93IGBDYW5ub3QgZmluZCBwcm9wIG9uIG9iamVjdCAke2ZvdW5kT2JqZWN0LnR5cGVOYW1lfTogJHtmaXJzdE5hbWV9YDtcbiAgICB9XG4gICAgaWYgKHJldHVyblByb3Aua2luZCgpICE9IFwib2JqZWN0XCIgJiYgbG9va3VwLm5hbWVzLmxlbmd0aCA+IDEpIHtcbiAgICAgIHRocm93IGBZb3UgYXNrZWQgZm9yIHN1Yi1wcm9wZXJ0aWVzIG9mIGEgbm9uLW9iamVjdCB0eXBlIG9uICR7Zm91bmRPYmplY3QudHlwZU5hbWV9IHByb3BlcnR5ICR7Zmlyc3ROYW1lfWA7XG4gICAgfVxuICAgIGZvciAobGV0IGkgPSAxOyBpIDwgbG9va3VwLm5hbWVzLmxlbmd0aDsgaSsrKSB7XG4gICAgICBjb25zdCBsb29rdXBOYW1lID0gbG9va3VwLm5hbWVzW2ldO1xuICAgICAgY29uc3QgbG9va3VwUmVzdWx0ID0gcmV0dXJuUHJvcFtcInByb3BlcnRpZXNcIl0uZ2V0RW50cnkobG9va3VwTmFtZSk7XG4gICAgICBpZiAoIWxvb2t1cFJlc3VsdCkge1xuICAgICAgICB0aHJvdyBgQ2Fubm90IGZpbmQgcHJvcCBcIiR7bG9va3VwTmFtZX1cIiBvbiAke3JldHVyblByb3AubmFtZX1gO1xuICAgICAgfVxuXG4gICAgICBpZiAoaSAhPSBsb29rdXAubmFtZXMubGVuZ3RoIC0gMSAmJiBsb29rdXBSZXN1bHQua2luZCgpICE9IFwib2JqZWN0XCIpIHtcbiAgICAgICAgY29uc29sZS5sb2coe1xuICAgICAgICAgIGksXG4gICAgICAgICAgbGVuZ3RoOiBsb29rdXAubmFtZXMubGVuZ3RoLFxuICAgICAgICAgIGxvb2t1cE5hbWUsXG4gICAgICAgICAgbG9va3VwUmVzdWx0LFxuICAgICAgICB9KTtcbiAgICAgICAgdGhyb3cgYENhbm5vdCBsb29rIHVwIGEgc3ViLXByb3BlcnR5IG9mIGEgbm9uIG9iamVjdCBQcm9wOiAke1xuICAgICAgICAgIGZvdW5kT2JqZWN0LnR5cGVOYW1lXG4gICAgICAgIH0gcHJvcGVydHkgJHtsb29rdXBOYW1lfSBpcyAke2xvb2t1cFJlc3VsdC5raW5kKCl9YDtcbiAgICAgIH1cblxuICAgICAgcmV0dXJuUHJvcCA9IGxvb2t1cFJlc3VsdDtcbiAgICB9XG4gICAgcmV0dXJuIHJldHVyblByb3A7XG4gIH1cblxuICAvLyBUaGVzZSBhcmUgXCJiYXNpY1wiIG9iamVjdHMgLSB0aGV5IGRvbid0IGhhdmUgYW55IGV4dHJhIGJlaGF2aW9yIG9yXG4gIC8vIGF1dG9tYXRpYyBmaWVsZHMuIFRoZXkganVzdCBzdG9yZSB0aGUgZmllbGRzIHlvdSBnaXZlIHRoZW0uXG4gIGJhc2UoY29uc3RydWN0b3JBcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpOiBCYXNlT2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBCYXNlT2JqZWN0KGNvbnN0cnVjdG9yQXJncyk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cblxuICAvLyBUaGVzZSBhcmUgXCJzeXN0ZW1cIiBvYmplY3RzIC0gdGhleSBoYXZlIHdoYXQgaXMgbmVlZGVkIHRvIGJlIGFuIG9iamVjdFxuICAvLyBpbnNpZGUgb3VyIHN5c3RlbS4gVGhleSBjb21lIHdpdGggdGhpbmdzIGxpa2UgdHlwZXMsIElEcywgdGVuYW5jeSxcbiAgLy8gZXRjLlxuICBzeXN0ZW0oY29uc3RydWN0b3JBcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpOiBTeXN0ZW1PYmplY3Qge1xuICAgIGNvbnN0IGNvbXB5ID0gbmV3IFN5c3RlbU9iamVjdChjb25zdHJ1Y3RvckFyZ3MpO1xuICAgIHRoaXMub2JqZWN0cy5wdXNoKGNvbXB5KTtcbiAgICBpZiAoY29uc3RydWN0b3JBcmdzLm9wdGlvbnMpIHtcbiAgICAgIGNvbnN0cnVjdG9yQXJncy5vcHRpb25zKGNvbXB5KTtcbiAgICB9XG4gICAgcmV0dXJuIGNvbXB5O1xuICB9XG5cbiAgY29tcG9uZW50KGNvbnN0cnVjdG9yQXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKTogQ29tcG9uZW50T2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBDb21wb25lbnRPYmplY3QoY29uc3RydWN0b3JBcmdzKTtcbiAgICB0aGlzLm9iamVjdHMucHVzaChjb21weSk7XG4gICAgaWYgKGNvbnN0cnVjdG9yQXJncy5vcHRpb25zKSB7XG4gICAgICBjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucyhjb21weSk7XG4gICAgfVxuICAgIHJldHVybiBjb21weTtcbiAgfVxuXG4gIGVudGl0eShjb25zdHJ1Y3RvckFyZ3M6IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcik6IEVudGl0eU9iamVjdCB7XG4gICAgY29uc3QgY29tcHkgPSBuZXcgRW50aXR5T2JqZWN0KGNvbnN0cnVjdG9yQXJncyk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cblxuICBjb21wb25lbnRBbmRFbnRpdHkoXG4gICAgY29uc3RydWN0b3JBcmdzOiBDb21wb25lbnRBbmRFbnRpdHlPYmplY3RDb25zdHJ1Y3RvcixcbiAgKTogQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBDb21wb25lbnRBbmRFbnRpdHlPYmplY3QoY29uc3RydWN0b3JBcmdzKTtcbiAgICB0aGlzLm9iamVjdHMucHVzaChjb21weS5jb21wb25lbnQpO1xuICAgIHRoaXMub2JqZWN0cy5wdXNoKGNvbXB5LmVudGl0eSk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkuZW50aXR5RXZlbnQpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cbn1cblxuZXhwb3J0IGNvbnN0IHJlZ2lzdHJ5ID0gbmV3IFJlZ2lzdHJ5KCk7XG4iXX0=