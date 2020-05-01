#!/usr/bin/env node
"use strict";

var _chalk = _interopRequireDefault(require("chalk"));

var _figlet = _interopRequireDefault(require("figlet"));

var _path = _interopRequireDefault(require("path"));

var _commander = _interopRequireDefault(require("commander"));

var _registry = require("src/registry");

var _protobuf = require("src/codegen/protobuf");

var _rust = require("src/codegen/rust");

var _listr = _interopRequireDefault(require("listr"));

require("src/loader");

var _fs = _interopRequireDefault(require("fs"));

var _util = require("util");

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { "default": obj }; }

function asyncGeneratorStep(gen, resolve, reject, _next, _throw, key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { Promise.resolve(value).then(_next, _throw); } }

function _asyncToGenerator(fn) { return function () { var self = this, args = arguments; return new Promise(function (resolve, reject) { var gen = fn.apply(self, args); function _next(value) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "next", value); } function _throw(err) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "throw", err); } _next(undefined); }); }; }

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

//import childProcess from "child_process";
//import util from "util";
//const execCmd = util.promisify(childProcess.exec);
console.log(_chalk["default"].greenBright(_figlet["default"].textSync("Lets go!", {
  horizontalLayout: "full"
})));

_commander["default"].version("0.0.1").description("Code Generation to rule them all").option("-v, --verbose", "show verbose output").parse(process.argv);

main(_commander["default"]);

function main(program) {
  // @ts-ignore
  var renderer;

  if (program.verbose) {
    renderer = "verbose";
  } else {
    renderer = "default";
  }

  var tasks = new _listr["default"]([{
    title: "Generating ".concat(_chalk["default"].keyword("darkseagreen")("Protobuf")),
    task: function task() {
      return generateProtobuf();
    }
  }, {
    title: "Generating ".concat(_chalk["default"].keyword("orange")("Rust")),
    task: function task() {
      return generateRust();
    }
  }], {
    renderer: renderer,
    concurrent: true
  });
  tasks.run()["catch"](function (err) {
    console.log(err);
  });
}

function generateProtobuf() {
  var tasks = [];

  var _iterator = _createForOfIteratorHelper(_registry.registry.serviceNames()),
      _step;

  try {
    var _loop = function _loop() {
      var serviceName = _step.value;
      tasks.push({
        title: "Protobuf Service ".concat(_chalk["default"].keyword("darkseagreen")(serviceName)),
        task: function () {
          var _task = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee() {
            var cp, protoFile, writeFileAsync;
            return regeneratorRuntime.wrap(function _callee$(_context) {
              while (1) {
                switch (_context.prev = _context.next) {
                  case 0:
                    cp = new _protobuf.ProtobufFormatter(_registry.registry.getObjectsForServiceName(serviceName));
                    protoFile = _path["default"].join(__dirname, "..", "..", "proto", "si.".concat(serviceName, ".proto"));
                    writeFileAsync = (0, _util.promisify)(_fs["default"].writeFile);
                    _context.next = 5;
                    return writeFileAsync(protoFile, cp.generateString());

                  case 5:
                  case "end":
                    return _context.stop();
                }
              }
            }, _callee);
          }));

          function task() {
            return _task.apply(this, arguments);
          }

          return task;
        }()
      });
    };

    for (_iterator.s(); !(_step = _iterator.n()).done;) {
      _loop();
    }
  } catch (err) {
    _iterator.e(err);
  } finally {
    _iterator.f();
  }

  return new _listr["default"](tasks, {
    concurrent: true
  });
}

function generateRust() {
  var tasks = [];

  var _iterator2 = _createForOfIteratorHelper(_registry.registry.serviceNames()),
      _step2;

  try {
    var _loop2 = function _loop2() {
      var serviceName = _step2.value;
      var codegenRust = new _rust.CodegenRust(serviceName);

      var systemObjects = _registry.registry.getObjectsForServiceName(serviceName);

      tasks.push({
        title: "Rust service ".concat(_chalk["default"].keyword("orange")("gen/service.rs"), " for ").concat(serviceName),
        task: function () {
          var _task2 = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee2() {
            return regeneratorRuntime.wrap(function _callee2$(_context2) {
              while (1) {
                switch (_context2.prev = _context2.next) {
                  case 0:
                    _context2.next = 2;
                    return codegenRust.generateGenService();

                  case 2:
                  case "end":
                    return _context2.stop();
                }
              }
            }, _callee2);
          }));

          function task() {
            return _task2.apply(this, arguments);
          }

          return task;
        }()
      });

      if (systemObjects.some(function (o) {
        return o.kind() != "baseObject";
      })) {
        tasks.push({
          title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/mod.rs"), " for ").concat(serviceName),
          task: function () {
            var _task3 = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee3() {
              return regeneratorRuntime.wrap(function _callee3$(_context3) {
                while (1) {
                  switch (_context3.prev = _context3.next) {
                    case 0:
                      _context3.next = 2;
                      return codegenRust.generateGenMod();

                    case 2:
                    case "end":
                      return _context3.stop();
                  }
                }
              }, _callee3);
            }));

            function task() {
              return _task3.apply(this, arguments);
            }

            return task;
          }()
        });
        tasks.push({
          title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/model/mod.rs"), " for ").concat(serviceName),
          task: function () {
            var _task4 = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee4() {
              return regeneratorRuntime.wrap(function _callee4$(_context4) {
                while (1) {
                  switch (_context4.prev = _context4.next) {
                    case 0:
                      _context4.next = 2;
                      return codegenRust.generateGenModelMod();

                    case 2:
                    case "end":
                      return _context4.stop();
                  }
                }
              }, _callee4);
            }));

            function task() {
              return _task4.apply(this, arguments);
            }

            return task;
          }()
        });

        var _iterator3 = _createForOfIteratorHelper(_registry.registry.getObjectsForServiceName(serviceName)),
            _step3;

        try {
          var _loop3 = function _loop3() {
            var systemObject = _step3.value;

            if (systemObject.kind() != "baseObject") {
              tasks.push({
                title: "Rust model ".concat(_chalk["default"].keyword("orange")(serviceName), " ").concat(systemObject.typeName),
                task: function () {
                  var _task6 = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee6() {
                    return regeneratorRuntime.wrap(function _callee6$(_context6) {
                      while (1) {
                        switch (_context6.prev = _context6.next) {
                          case 0:
                            _context6.next = 2;
                            return codegenRust.generateGenModel(systemObject);

                          case 2:
                          case "end":
                            return _context6.stop();
                        }
                      }
                    }, _callee6);
                  }));

                  function task() {
                    return _task6.apply(this, arguments);
                  }

                  return task;
                }()
              });
            }
          };

          for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
            _loop3();
          }
        } catch (err) {
          _iterator3.e(err);
        } finally {
          _iterator3.f();
        }

        tasks.push({
          title: "Rust format ".concat(serviceName),
          task: function () {
            var _task5 = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee5() {
              return regeneratorRuntime.wrap(function _callee5$(_context5) {
                while (1) {
                  switch (_context5.prev = _context5.next) {
                    case 0:
                      _context5.next = 2;
                      return codegenRust.formatCode();

                    case 2:
                    case "end":
                      return _context5.stop();
                  }
                }
              }, _callee5);
            }));

            function task() {
              return _task5.apply(this, arguments);
            }

            return task;
          }()
        });
      }
    };

    for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
      _loop2();
    }
  } catch (err) {
    _iterator2.e(err);
  } finally {
    _iterator2.f();
  }

  return new _listr["default"](tasks, {
    concurrent: false
  });
}
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9iaW4vc2ktZ2VuZXJhdGUudHMiXSwibmFtZXMiOlsiY29uc29sZSIsImxvZyIsImNoYWxrIiwiZ3JlZW5CcmlnaHQiLCJmaWdsZXQiLCJ0ZXh0U3luYyIsImhvcml6b250YWxMYXlvdXQiLCJwcm9ncmFtIiwidmVyc2lvbiIsImRlc2NyaXB0aW9uIiwib3B0aW9uIiwicGFyc2UiLCJwcm9jZXNzIiwiYXJndiIsIm1haW4iLCJyZW5kZXJlciIsInZlcmJvc2UiLCJ0YXNrcyIsIkxpc3RyIiwidGl0bGUiLCJrZXl3b3JkIiwidGFzayIsImdlbmVyYXRlUHJvdG9idWYiLCJnZW5lcmF0ZVJ1c3QiLCJjb25jdXJyZW50IiwicnVuIiwiZXJyIiwicmVnaXN0cnkiLCJzZXJ2aWNlTmFtZXMiLCJzZXJ2aWNlTmFtZSIsInB1c2giLCJjcCIsIlByb3RvYnVmRm9ybWF0dGVyIiwiZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lIiwicHJvdG9GaWxlIiwicGF0aCIsImpvaW4iLCJfX2Rpcm5hbWUiLCJ3cml0ZUZpbGVBc3luYyIsImZzIiwid3JpdGVGaWxlIiwiZ2VuZXJhdGVTdHJpbmciLCJjb2RlZ2VuUnVzdCIsIkNvZGVnZW5SdXN0Iiwic3lzdGVtT2JqZWN0cyIsImdlbmVyYXRlR2VuU2VydmljZSIsInNvbWUiLCJvIiwia2luZCIsImdlbmVyYXRlR2VuTW9kIiwiZ2VuZXJhdGVHZW5Nb2RlbE1vZCIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwiZ2VuZXJhdGVHZW5Nb2RlbCIsImZvcm1hdENvZGUiXSwibWFwcGluZ3MiOiJBQUFBOzs7QUFFQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7Ozs7Ozs7Ozs7Ozs7QUFDQTtBQUNBO0FBQ0E7QUFFQUEsT0FBTyxDQUFDQyxHQUFSLENBQ0VDLGtCQUFNQyxXQUFOLENBQWtCQyxtQkFBT0MsUUFBUCxDQUFnQixVQUFoQixFQUE0QjtBQUFF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RzdCLDJEQUFtRDtBQUFBO0FBc0RsRDtBQXpENEI7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUEyRDdCLFNBQU8sSUFBSXBDLGlCQUFKLENBQVVELEtBQVYsRUFBaUI7QUFBRU8sSUFBQUEsVUFBVSxFQUFFO0FBQWQsR0FBakIsQ0FBUDtBQUNEIiwic291cmNlc0NvbnRlbnQiOlsiIyEvdXNyL2Jpbi9lbnYgbm9kZVxuXG5pbXBvcnQgY2hhbGsgZnJvbSBcImNoYWxrXCI7XG5pbXBvcnQgZmlnbGV0IGZyb20gXCJmaWdsZXRcIjtcbmltcG9ydCBwYXRoIGZyb20gXCJwYXRoXCI7XG5pbXBvcnQgcHJvZ3JhbSBmcm9tIFwiY29tbWFuZGVyXCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCJzcmMvcmVnaXN0cnlcIjtcbmltcG9ydCB7IFByb3RvYnVmRm9ybWF0dGVyIH0gZnJvbSBcInNyYy9jb2RlZ2VuL3Byb3RvYnVmXCI7XG5pbXBvcnQgeyBDb2RlZ2VuUnVzdCB9IGZyb20gXCJzcmMvY29kZWdlbi9ydXN0XCI7XG5pbXBvcnQgTGlzdHIsIHsgTGlzdHJSZW5kZXJlclZhbHVlIH0gZnJvbSBcImxpc3RyXCI7XG5pbXBvcnQgXCJzcmMvbG9hZGVyXCI7XG5pbXBvcnQgZnMgZnJvbSBcImZzXCI7XG5pbXBvcnQgeyBwcm9taXNpZnkgfSBmcm9tIFwidXRpbFwiO1xuLy9pbXBvcnQgY2hpbGRQcm9jZXNzIGZyb20gXCJjaGlsZF9wcm9jZXNzXCI7XG4vL2ltcG9ydCB1dGlsIGZyb20gXCJ1dGlsXCI7XG4vL2NvbnN0IGV4ZWNDbWQgPSB1dGlsLnByb21pc2lmeShjaGlsZFByb2Nlc3MuZXhlYyk7XG5cbmNvbnNvbGUubG9nKFxuICBjaGFsay5ncmVlbkJyaWdodChmaWdsZXQudGV4dFN5bmMoXCJMZXRzIGdvIVwiLCB7IGhvcml6b250YWxMYXlvdXQ6IFwiZnVsbFwiIH0pKSxcbik7XG5cbnByb2dyYW1cbiAgLnZlcnNpb24oXCIwLjAuMVwiKVxuICAuZGVzY3JpcHRpb24oXCJDb2RlIEdlbmVyYXRpb24gdG8gcnVsZSB0aGVtIGFsbFwiKVxuICAub3B0aW9uKFwiLXYsIC0tdmVyYm9zZVwiLCBcInNob3cgdmVyYm9zZSBvdXRwdXRcIilcbiAgLnBhcnNlKHByb2Nlc3MuYXJndik7XG5cbm1haW4ocHJvZ3JhbSk7XG5cbmZ1bmN0aW9uIG1haW4ocHJvZ3JhbTogcHJvZ3JhbS5Db21tYW5kKTogdm9pZCB7XG4gIC8vIEB0cy1pZ25vcmVcbiAgbGV0IHJlbmRlcmVyOiBMaXN0clJlbmRlcmVyVmFsdWU8YW55PjtcbiAgaWYgKHByb2dyYW0udmVyYm9zZSkge1xuICAgIHJlbmRlcmVyID0gXCJ2ZXJib3NlXCI7XG4gIH0gZWxzZSB7XG4gICAgcmVuZGVyZXIgPSBcImRlZmF1bHRcIjtcbiAgfVxuICBjb25zdCB0YXNrcyA9IG5ldyBMaXN0cihcbiAgICBbXG4gICAgICB7XG4gICAgICAgIHRpdGxlOiBgR2VuZXJhdGluZyAke2NoYWxrLmtleXdvcmQoXCJkYXJrc2VhZ3JlZW5cIikoXCJQcm90b2J1ZlwiKX1gLFxuICAgICAgICB0YXNrOiAoKTogTGlzdHIgPT4ge1xuICAgICAgICAgIHJldHVybiBnZW5lcmF0ZVByb3RvYnVmKCk7XG4gICAgICAgIH0sXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICB0aXRsZTogYEdlbmVyYXRpbmcgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKFwiUnVzdFwiKX1gLFxuICAgICAgICB0YXNrOiAoKTogTGlzdHIgPT4ge1xuICAgICAgICAgIHJldHVybiBnZW5lcmF0ZVJ1c3QoKTtcbiAgICAgICAgfSxcbiAgICAgIH0sXG4gICAgXSxcbiAgICB7XG4gICAgICByZW5kZXJlcixcbiAgICAgIGNvbmN1cnJlbnQ6IHRydWUsXG4gICAgfSxcbiAgKTtcbiAgdGFza3MucnVuKCkuY2F0Y2goKGVycjogRXJyb3IpOiB2b2lkID0+IHtcbiAgICBjb25zb2xlLmxvZyhlcnIpO1xuICB9KTtcbn1cblxuZnVuY3Rpb24gZ2VuZXJhdGVQcm90b2J1ZigpOiBMaXN0ciB7XG4gIGNvbnN0IHRhc2tzID0gW107XG4gIGZvciAoY29uc3Qgc2VydmljZU5hbWUgb2YgcmVnaXN0cnkuc2VydmljZU5hbWVzKCkpIHtcbiAgICB0YXNrcy5wdXNoKHtcbiAgICAgIHRpdGxlOiBgUHJvdG9idWYgU2VydmljZSAke2NoYWxrLmtleXdvcmQoXCJkYXJrc2VhZ3JlZW5cIikoc2VydmljZU5hbWUpfWAsXG4gICAgICB0YXNrOiBhc3luYyAoKSA9PiB7XG4gICAgICAgIGNvbnN0IGNwID0gbmV3IFByb3RvYnVmRm9ybWF0dGVyKFxuICAgICAgICAgIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSksXG4gICAgICAgICk7XG4gICAgICAgIGNvbnN0IHByb3RvRmlsZSA9IHBhdGguam9pbihcbiAgICAgICAgICBfX2Rpcm5hbWUsXG4gICAgICAgICAgXCIuLlwiLFxuICAgICAgICAgIFwiLi5cIixcbiAgICAgICAgICBcInByb3RvXCIsXG4gICAgICAgICAgYHNpLiR7c2VydmljZU5hbWV9LnByb3RvYCxcbiAgICAgICAgKTtcbiAgICAgICAgY29uc3Qgd3JpdGVGaWxlQXN5bmMgPSBwcm9taXNpZnkoZnMud3JpdGVGaWxlKTtcbiAgICAgICAgYXdhaXQgd3JpdGVGaWxlQXN5bmMocHJvdG9GaWxlLCBjcC5nZW5lcmF0ZVN0cmluZygpKTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cbiAgcmV0dXJuIG5ldyBMaXN0cih0YXNrcywgeyBjb25jdXJyZW50OiB0cnVlIH0pO1xufVxuXG5mdW5jdGlvbiBnZW5lcmF0ZVJ1c3QoKTogTGlzdHIge1xuICBjb25zdCB0YXNrcyA9IFtdO1xuXG4gIGZvciAoY29uc3Qgc2VydmljZU5hbWUgb2YgcmVnaXN0cnkuc2VydmljZU5hbWVzKCkpIHtcbiAgICBjb25zdCBjb2RlZ2VuUnVzdCA9IG5ldyBDb2RlZ2VuUnVzdChzZXJ2aWNlTmFtZSk7XG4gICAgY29uc3Qgc3lzdGVtT2JqZWN0cyA9IHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSk7XG5cbiAgICB0YXNrcy5wdXNoKHtcbiAgICAgIHRpdGxlOiBgUnVzdCBzZXJ2aWNlICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcbiAgICAgICAgXCJnZW4vc2VydmljZS5yc1wiLFxuICAgICAgKX0gZm9yICR7c2VydmljZU5hbWV9YCxcbiAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5TZXJ2aWNlKCk7XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgaWYgKHN5c3RlbU9iamVjdHMuc29tZShvID0+IG8ua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKSkge1xuICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgIHRpdGxlOiBgUnVzdCAke2NoYWxrLmtleXdvcmQoXCJvcmFuZ2VcIikoXG4gICAgICAgICAgXCJnZW4vbW9kLnJzXCIsXG4gICAgICAgICl9IGZvciAke3NlcnZpY2VOYW1lfWAsXG4gICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICBhd2FpdCBjb2RlZ2VuUnVzdC5nZW5lcmF0ZUdlbk1vZCgpO1xuICAgICAgICB9LFxuICAgICAgfSk7XG5cbiAgICAgIHRhc2tzLnB1c2goe1xuICAgICAgICB0aXRsZTogYFJ1c3QgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKFxuICAgICAgICAgIFwiZ2VuL21vZGVsL21vZC5yc1wiLFxuICAgICAgICApfSBmb3IgJHtzZXJ2aWNlTmFtZX1gLFxuICAgICAgICB0YXNrOiBhc3luYyAoKTogUHJvbWlzZTx2b2lkPiA9PiB7XG4gICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5Nb2RlbE1vZCgpO1xuICAgICAgICB9LFxuICAgICAgfSk7XG5cbiAgICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqZWN0IG9mIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShcbiAgICAgICAgc2VydmljZU5hbWUsXG4gICAgICApKSB7XG4gICAgICAgIGlmIChzeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKSB7XG4gICAgICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgICAgICB0aXRsZTogYFJ1c3QgbW9kZWwgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKHNlcnZpY2VOYW1lKX0gJHtcbiAgICAgICAgICAgICAgc3lzdGVtT2JqZWN0LnR5cGVOYW1lXG4gICAgICAgICAgICB9YCxcbiAgICAgICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5Nb2RlbChzeXN0ZW1PYmplY3QpO1xuICAgICAgICAgICAgfSxcbiAgICAgICAgICB9KTtcbiAgICAgICAgfVxuICAgICAgfVxuXG4gICAgICB0YXNrcy5wdXNoKHtcbiAgICAgICAgdGl0bGU6IGBSdXN0IGZvcm1hdCAke3NlcnZpY2VOYW1lfWAsXG4gICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICBhd2FpdCBjb2RlZ2VuUnVzdC5mb3JtYXRDb2RlKCk7XG4gICAgICAgIH0sXG4gICAgICB9KTtcbiAgICB9XG4gIH1cblxuICByZXR1cm4gbmV3IExpc3RyKHRhc2tzLCB7IGNvbmN1cnJlbnQ6IGZhbHNlIH0pO1xufVxuIl19