"use strict";

var _interopRequireWildcard = require("@babel/runtime/helpers/interopRequireWildcard");

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

var _regenerator = _interopRequireDefault(require("@babel/runtime/regenerator"));

var _asyncToGenerator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncToGenerator"));

var _chalk = _interopRequireDefault(require("chalk"));

var _figlet = _interopRequireDefault(require("figlet"));

var _path = _interopRequireDefault(require("path"));

var _commander = _interopRequireDefault(require("commander"));

var _registry = require("../registry");

var _protobuf = require("../codegen/protobuf");

var _rust = require("../codegen/rust");

var _listr = _interopRequireDefault(require("listr"));

require("../loader");

var _fs = _interopRequireDefault(require("fs"));

var _util = _interopRequireWildcard(require("util"));

var _child_process = _interopRequireDefault(require("child_process"));

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

var execCmd = _util["default"].promisify(_child_process["default"].exec);

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
  }, {
    title: "Generating ".concat(_chalk["default"].keyword("yellow")("Javascript Library")),
    task: function task() {
      return generateJavascriptLibrary();
    }
  }], {
    renderer: renderer,
    concurrent: true
  });
  tasks.run()["catch"](function (err) {
    console.log(err);
  });
}

function generateJavascriptLibrary() {
  var tasks = [];
  tasks.push({
    title: "Javascript library for si-registry",
    task: function () {
      var _task = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee() {
        return _regenerator["default"].wrap(function _callee$(_context) {
          while (1) {
            switch (_context.prev = _context.next) {
              case 0:
                _context.next = 2;
                return execCmd("npm run build");

              case 2:
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
  return new _listr["default"](tasks, {
    concurrent: true
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
          var _task2 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee2() {
            var cp, protoFile, writeFileAsync;
            return _regenerator["default"].wrap(function _callee2$(_context2) {
              while (1) {
                switch (_context2.prev = _context2.next) {
                  case 0:
                    cp = new _protobuf.ProtobufFormatter(_registry.registry.getObjectsForServiceName(serviceName));
                    protoFile = _path["default"].join("./proto", "si.".concat(serviceName, ".proto"));
                    writeFileAsync = (0, _util.promisify)(_fs["default"].writeFile);
                    _context2.next = 5;
                    return writeFileAsync(protoFile, cp.generateString());

                  case 5:
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

      if (systemObjects.some(function (o) {
        return o.kind() != "baseObject";
      })) {
        tasks.push({
          title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/mod.rs"), " for ").concat(serviceName),
          task: function () {
            var _task3 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee3() {
              return _regenerator["default"].wrap(function _callee3$(_context3) {
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

        if (codegenRust.hasServiceMethods()) {
          tasks.push({
            title: "Rust service ".concat(_chalk["default"].keyword("orange")("gen/service.rs"), " for ").concat(serviceName),
            task: function () {
              var _task4 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee4() {
                return _regenerator["default"].wrap(function _callee4$(_context4) {
                  while (1) {
                    switch (_context4.prev = _context4.next) {
                      case 0:
                        _context4.next = 2;
                        return codegenRust.generateGenService();

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
        }

        if (codegenRust.hasModels()) {
          tasks.push({
            title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/model/mod.rs"), " for ").concat(serviceName),
            task: function () {
              var _task5 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee5() {
                return _regenerator["default"].wrap(function _callee5$(_context5) {
                  while (1) {
                    switch (_context5.prev = _context5.next) {
                      case 0:
                        _context5.next = 2;
                        return codegenRust.generateGenModelMod();

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

          var _iterator3 = _createForOfIteratorHelper(_registry.registry.getObjectsForServiceName(serviceName)),
              _step3;

          try {
            var _loop3 = function _loop3() {
              var systemObject = _step3.value;

              if (systemObject.kind() != "baseObject") {
                tasks.push({
                  title: "Rust model ".concat(_chalk["default"].keyword("orange")(serviceName), " ").concat(systemObject.typeName),
                  task: function () {
                    var _task6 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee6() {
                      return _regenerator["default"].wrap(function _callee6$(_context6) {
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
        }

        if (codegenRust.hasEntityIntegrationServcices()) {
          tasks.push({
            title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/agent/mod.rs"), " for ").concat(serviceName),
            task: function () {
              var _task7 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee7() {
                return _regenerator["default"].wrap(function _callee7$(_context7) {
                  while (1) {
                    switch (_context7.prev = _context7.next) {
                      case 0:
                        _context7.next = 2;
                        return codegenRust.generateGenAgentMod();

                      case 2:
                      case "end":
                        return _context7.stop();
                    }
                  }
                }, _callee7);
              }));

              function task() {
                return _task7.apply(this, arguments);
              }

              return task;
            }()
          });

          var _iterator4 = _createForOfIteratorHelper(codegenRust.entityIntegrationServices()),
              _step4;

          try {
            var _loop4 = function _loop4() {
              var agent = _step4.value;
              tasks.push({
                title: "Rust agent ".concat(_chalk["default"].keyword("orange")(serviceName), " ").concat(agent.agentName),
                task: function () {
                  var _task8 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee8() {
                    return _regenerator["default"].wrap(function _callee8$(_context8) {
                      while (1) {
                        switch (_context8.prev = _context8.next) {
                          case 0:
                            _context8.next = 2;
                            return codegenRust.generateGenAgent(agent);

                          case 2:
                          case "end":
                            return _context8.stop();
                        }
                      }
                    }, _callee8);
                  }));

                  function task() {
                    return _task8.apply(this, arguments);
                  }

                  return task;
                }()
              });
            };

            for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
              _loop4();
            }
          } catch (err) {
            _iterator4.e(err);
          } finally {
            _iterator4.f();
          }
        }

        tasks.push({
          title: "Rust format ".concat(serviceName),
          task: function () {
            var _task9 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee9() {
              return _regenerator["default"].wrap(function _callee9$(_context9) {
                while (1) {
                  switch (_context9.prev = _context9.next) {
                    case 0:
                      _context9.next = 2;
                      return codegenRust.formatCode();

                    case 2:
                    case "end":
                      return _context9.stop();
                  }
                }
              }, _callee9);
            }));

            function task() {
              return _task9.apply(this, arguments);
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9iaW4vc2ktZ2VuZXJhdGUudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiY29uc29sZSIsImxvZyIsImNoYWxrIiwiZ3JlZW5CcmlnaHQiLCJmaWdsZXQiLCJ0ZXh0U3luYyIsImhvcml6b250YWxMYXlvdXQiLCJwcm9ncmFtIiwidmVyc2lvbiIsImRlc2NyaXB0aW9uIiwib3B0aW9uIiwicGFyc2UiLCJwcm9jZXNzIiwiYXJndiIsIm1haW4iLCJyZW5kZXJlciIsInZlcmJvc2UiLCJ0YXNrcyIsIkxpc3RyIiwidGl0bGUiLCJrZXl3b3JkIiwidGFzayIsImdlbmVyYXRlUHJvdG9idWYiLCJnZW5lcmF0ZVJ1c3QiLCJnZW5lcmF0ZUphdmFzY3JpcHRMaWJyYXJ5IiwiY29uY3VycmVudCIsInJ1biIsImVyciIsInB1c2giLCJyZWdpc3RyeSIsInNlcnZpY2VOYW1lcyIsInNlcnZpY2VOYW1lIiwiY3AiLCJQcm90b2J1ZkZvcm1hdHRlciIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsInByb3RvRmlsZSIsInBhdGgiLCJqb2luIiwid3JpdGVGaWxlQXN5bmMiLCJmcyIsIndyaXRlRmlsZSIsImdlbmVyYXRlU3RyaW5nIiwiY29kZWdlblJ1c3QiLCJDb2RlZ2VuUnVzdCIsInN5c3RlbU9iamVjdHMiLCJzb21lIiwibyIsImtpbmQiLCJnZW5lcmF0ZUdlbk1vZCIsImhhc1NlcnZpY2VNZXRob2RzIiwiZ2VuZXJhdGVHZW5TZXJ2aWNlIiwiaGFzTW9kZWxzIiwiZ2VuZXJhdGVHZW5Nb2RlbE1vZCIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwiZ2VuZXJhdGVHZW5Nb2RlbCIsImhhc0VudGl0eUludGVncmF0aW9uU2VydmNpY2VzIiwiZ2VuZXJhdGVHZW5BZ2VudE1vZCIsImVudGl0eUludGVncmF0aW9uU2VydmljZXMiLCJhZ2VudCIsImFnZW50TmFtZSIsImdlbmVy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V0FBVyxDQUFDZ0IseUJBQVosRUFWMkI7QUFBQTs7QUFBQTtBQUFBO0FBQUEsa0JBVXBDQyxLQVZvQztBQVc3QzFDLGNBQUFBLEtBQUssQ0FBQ1csSUFBTixDQUFXO0FBQ1RULGdCQUFBQSxLQUFLLHVCQUFnQmpCLGtCQUFNa0IsT0FBTixDQUFjLFFBQWQsRUFBd0JXLFdBQXhCLENBQWhCLGNBQ0g0QixLQUFLLENBQUNDLFNBREgsQ0FESTtBQUlUdkMsZ0JBQUFBLElBQUk7QUFBQSw2R0FBRTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQSxtQ0FDRXFCLFdBQVcsQ0FBQ21CLGdCQUFaLENBQTZCRixLQUE3QixDQURGOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLG1CQUFGOztBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBSkssZUFBWDtBQVg2Qzs7QUFVL0MsbUVBQTZEO0FBQUE7QUFTNUQ7QUFuQjhDO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFvQmhEOztBQUVEMUMsUUFBQUEsS0FBSyxDQUFDVyxJQUFOLENBQVc7QUFDVFQsVUFBQUEsS0FBSyx3QkFBaUJZLFdBQWpCLENBREk7QUFFVFYsVUFBQUEsSUFBSTtBQUFBLHVHQUFFO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLDZCQUNFcUIsV0FBVyxDQUFDb0IsVUFBWixFQURGOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLGFBQUY7O0FBQUE7QUFBQTtBQUFBOztBQUFBO0FBQUE7QUFGSyxTQUFYO0FBTUQ7QUFsRjBCOztBQUc3QiwyREFBbUQ7QUFBQTtBQWdGbEQ7QUFuRjRCO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBcUY3QixTQUFPLElBQUk1QyxpQkFBSixDQUFVRCxLQUFWLEVBQWlCO0FBQUVRLElBQUFBLFVBQVUsRUFBRTtBQUFkLEdBQWpCLENBQVA7QUFDRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCBjaGFsayBmcm9tIFwiY2hhbGtcIjtcbmltcG9ydCBmaWdsZXQgZnJvbSBcImZpZ2xldFwiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCBwcm9ncmFtIGZyb20gXCJjb21tYW5kZXJcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5pbXBvcnQgeyBQcm90b2J1ZkZvcm1hdHRlciB9IGZyb20gXCIuLi9jb2RlZ2VuL3Byb3RvYnVmXCI7XG5pbXBvcnQgeyBDb2RlZ2VuUnVzdCB9IGZyb20gXCIuLi9jb2RlZ2VuL3J1c3RcIjtcbmltcG9ydCBMaXN0ciwgeyBMaXN0clJlbmRlcmVyVmFsdWUgfSBmcm9tIFwibGlzdHJcIjtcbmltcG9ydCBcIi4uL2xvYWRlclwiO1xuaW1wb3J0IGZzIGZyb20gXCJmc1wiO1xuaW1wb3J0IHsgcHJvbWlzaWZ5IH0gZnJvbSBcInV0aWxcIjtcbmltcG9ydCBjaGlsZFByb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcbmltcG9ydCB1dGlsIGZyb20gXCJ1dGlsXCI7XG5jb25zdCBleGVjQ21kID0gdXRpbC5wcm9taXNpZnkoY2hpbGRQcm9jZXNzLmV4ZWMpO1xuXG5jb25zb2xlLmxvZyhcbiAgY2hhbGsuZ3JlZW5CcmlnaHQoZmlnbGV0LnRleHRTeW5jKFwiTGV0cyBnbyFcIiwgeyBob3Jpem9udGFsTGF5b3V0OiBcImZ1bGxcIiB9KSksXG4pO1xuXG5wcm9ncmFtXG4gIC52ZXJzaW9uKFwiMC4wLjFcIilcbiAgLmRlc2NyaXB0aW9uKFwiQ29kZSBHZW5lcmF0aW9uIHRvIHJ1bGUgdGhlbSBhbGxcIilcbiAgLm9wdGlvbihcIi12LCAtLXZlcmJvc2VcIiwgXCJzaG93IHZlcmJvc2Ugb3V0cHV0XCIpXG4gIC5wYXJzZShwcm9jZXNzLmFyZ3YpO1xuXG5tYWluKHByb2dyYW0pO1xuXG5mdW5jdGlvbiBtYWluKHByb2dyYW06IHByb2dyYW0uQ29tbWFuZCk6IHZvaWQge1xuICAvLyBAdHMtaWdub3JlXG4gIGxldCByZW5kZXJlcjogTGlzdHJSZW5kZXJlclZhbHVlPGFueT47XG4gIGlmIChwcm9ncmFtLnZlcmJvc2UpIHtcbiAgICByZW5kZXJlciA9IFwidmVyYm9zZVwiO1xuICB9IGVsc2Uge1xuICAgIHJlbmRlcmVyID0gXCJkZWZhdWx0XCI7XG4gIH1cbiAgY29uc3QgdGFza3MgPSBuZXcgTGlzdHIoXG4gICAgW1xuICAgICAge1xuICAgICAgICB0aXRsZTogYEdlbmVyYXRpbmcgJHtjaGFsay5rZXl3b3JkKFwiZGFya3NlYWdyZWVuXCIpKFwiUHJvdG9idWZcIil9YCxcbiAgICAgICAgdGFzazogKCk6IExpc3RyID0+IHtcbiAgICAgICAgICByZXR1cm4gZ2VuZXJhdGVQcm90b2J1ZigpO1xuICAgICAgICB9LFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgdGl0bGU6IGBHZW5lcmF0aW5nICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcIlJ1c3RcIil9YCxcbiAgICAgICAgdGFzazogKCk6IExpc3RyID0+IHtcbiAgICAgICAgICByZXR1cm4gZ2VuZXJhdGVSdXN0KCk7XG4gICAgICAgIH0sXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICB0aXRsZTogYEdlbmVyYXRpbmcgJHtjaGFsay5rZXl3b3JkKFwieWVsbG93XCIpKFwiSmF2YXNjcmlwdCBMaWJyYXJ5XCIpfWAsXG4gICAgICAgIHRhc2s6ICgpOiBMaXN0ciA9PiB7XG4gICAgICAgICAgcmV0dXJuIGdlbmVyYXRlSmF2YXNjcmlwdExpYnJhcnkoKTtcbiAgICAgICAgfSxcbiAgICAgIH0sXG4gICAgXSxcbiAgICB7XG4gICAgICByZW5kZXJlcixcbiAgICAgIGNvbmN1cnJlbnQ6IHRydWUsXG4gICAgfSxcbiAgKTtcbiAgdGFza3MucnVuKCkuY2F0Y2goKGVycjogRXJyb3IpOiB2b2lkID0+IHtcbiAgICBjb25zb2xlLmxvZyhlcnIpO1xuICB9KTtcbn1cblxuZnVuY3Rpb24gZ2VuZXJhdGVKYXZhc2NyaXB0TGlicmFyeSgpOiBMaXN0ciB7XG4gIGNvbnN0IHRhc2tzID0gW107XG4gIHRhc2tzLnB1c2goe1xuICAgIHRpdGxlOiBgSmF2YXNjcmlwdCBsaWJyYXJ5IGZvciBzaS1yZWdpc3RyeWAsXG4gICAgdGFzazogYXN5bmMgKCkgPT4ge1xuICAgICAgYXdhaXQgZXhlY0NtZChcIm5wbSBydW4gYnVpbGRcIik7XG4gICAgfSxcbiAgfSk7XG4gIHJldHVybiBuZXcgTGlzdHIodGFza3MsIHsgY29uY3VycmVudDogdHJ1ZSB9KTtcbn1cblxuZnVuY3Rpb24gZ2VuZXJhdGVQcm90b2J1ZigpOiBMaXN0ciB7XG4gIGNvbnN0IHRhc2tzID0gW107XG4gIGZvciAoY29uc3Qgc2VydmljZU5hbWUgb2YgcmVnaXN0cnkuc2VydmljZU5hbWVzKCkpIHtcbiAgICB0YXNrcy5wdXNoKHtcbiAgICAgIHRpdGxlOiBgUHJvdG9idWYgU2VydmljZSAke2NoYWxrLmtleXdvcmQoXCJkYXJrc2VhZ3JlZW5cIikoc2VydmljZU5hbWUpfWAsXG4gICAgICB0YXNrOiBhc3luYyAoKSA9PiB7XG4gICAgICAgIGNvbnN0IGNwID0gbmV3IFByb3RvYnVmRm9ybWF0dGVyKFxuICAgICAgICAgIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSksXG4gICAgICAgICk7XG4gICAgICAgIGNvbnN0IHByb3RvRmlsZSA9IHBhdGguam9pbihcIi4vcHJvdG9cIiwgYHNpLiR7c2VydmljZU5hbWV9LnByb3RvYCk7XG4gICAgICAgIGNvbnN0IHdyaXRlRmlsZUFzeW5jID0gcHJvbWlzaWZ5KGZzLndyaXRlRmlsZSk7XG4gICAgICAgIGF3YWl0IHdyaXRlRmlsZUFzeW5jKHByb3RvRmlsZSwgY3AuZ2VuZXJhdGVTdHJpbmcoKSk7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG4gIHJldHVybiBuZXcgTGlzdHIodGFza3MsIHsgY29uY3VycmVudDogdHJ1ZSB9KTtcbn1cblxuZnVuY3Rpb24gZ2VuZXJhdGVSdXN0KCk6IExpc3RyIHtcbiAgY29uc3QgdGFza3MgPSBbXTtcblxuICBmb3IgKGNvbnN0IHNlcnZpY2VOYW1lIG9mIHJlZ2lzdHJ5LnNlcnZpY2VOYW1lcygpKSB7XG4gICAgY29uc3QgY29kZWdlblJ1c3QgPSBuZXcgQ29kZWdlblJ1c3Qoc2VydmljZU5hbWUpO1xuICAgIGNvbnN0IHN5c3RlbU9iamVjdHMgPSByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoc2VydmljZU5hbWUpO1xuXG4gICAgaWYgKHN5c3RlbU9iamVjdHMuc29tZShvID0+IG8ua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKSkge1xuICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgIHRpdGxlOiBgUnVzdCAke2NoYWxrLmtleXdvcmQoXCJvcmFuZ2VcIikoXG4gICAgICAgICAgXCJnZW4vbW9kLnJzXCIsXG4gICAgICAgICl9IGZvciAke3NlcnZpY2VOYW1lfWAsXG4gICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICBhd2FpdCBjb2RlZ2VuUnVzdC5nZW5lcmF0ZUdlbk1vZCgpO1xuICAgICAgICB9LFxuICAgICAgfSk7XG5cbiAgICAgIGlmIChjb2RlZ2VuUnVzdC5oYXNTZXJ2aWNlTWV0aG9kcygpKSB7XG4gICAgICAgIHRhc2tzLnB1c2goe1xuICAgICAgICAgIHRpdGxlOiBgUnVzdCBzZXJ2aWNlICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcbiAgICAgICAgICAgIFwiZ2VuL3NlcnZpY2UucnNcIixcbiAgICAgICAgICApfSBmb3IgJHtzZXJ2aWNlTmFtZX1gLFxuICAgICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICAgIGF3YWl0IGNvZGVnZW5SdXN0LmdlbmVyYXRlR2VuU2VydmljZSgpO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfVxuXG4gICAgICBpZiAoY29kZWdlblJ1c3QuaGFzTW9kZWxzKCkpIHtcbiAgICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgICAgdGl0bGU6IGBSdXN0ICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcbiAgICAgICAgICAgIFwiZ2VuL21vZGVsL21vZC5yc1wiLFxuICAgICAgICAgICl9IGZvciAke3NlcnZpY2VOYW1lfWAsXG4gICAgICAgICAgdGFzazogYXN5bmMgKCk6IFByb21pc2U8dm9pZD4gPT4ge1xuICAgICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5Nb2RlbE1vZCgpO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuXG4gICAgICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqZWN0IG9mIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShcbiAgICAgICAgICBzZXJ2aWNlTmFtZSxcbiAgICAgICAgKSkge1xuICAgICAgICAgIGlmIChzeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKSB7XG4gICAgICAgICAgICB0YXNrcy5wdXNoKHtcbiAgICAgICAgICAgICAgdGl0bGU6IGBSdXN0IG1vZGVsICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShzZXJ2aWNlTmFtZSl9ICR7XG4gICAgICAgICAgICAgICAgc3lzdGVtT2JqZWN0LnR5cGVOYW1lXG4gICAgICAgICAgICAgIH1gLFxuICAgICAgICAgICAgICB0YXNrOiBhc3luYyAoKTogUHJvbWlzZTx2b2lkPiA9PiB7XG4gICAgICAgICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5Nb2RlbChzeXN0ZW1PYmplY3QpO1xuICAgICAgICAgICAgICB9LFxuICAgICAgICAgICAgfSk7XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICB9XG5cbiAgICAgIGlmIChjb2RlZ2VuUnVzdC5oYXNFbnRpdHlJbnRlZ3JhdGlvblNlcnZjaWNlcygpKSB7XG4gICAgICAgIHRhc2tzLnB1c2goe1xuICAgICAgICAgIHRpdGxlOiBgUnVzdCAke2NoYWxrLmtleXdvcmQoXCJvcmFuZ2VcIikoXG4gICAgICAgICAgICBcImdlbi9hZ2VudC9tb2QucnNcIixcbiAgICAgICAgICApfSBmb3IgJHtzZXJ2aWNlTmFtZX1gLFxuICAgICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICAgIGF3YWl0IGNvZGVnZW5SdXN0LmdlbmVyYXRlR2VuQWdlbnRNb2QoKTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcblxuICAgICAgICBmb3IgKGNvbnN0IGFnZW50IG9mIGNvZGVnZW5SdXN0LmVudGl0eUludGVncmF0aW9uU2VydmljZXMoKSkge1xuICAgICAgICAgIHRhc2tzLnB1c2goe1xuICAgICAgICAgICAgdGl0bGU6IGBSdXN0IGFnZW50ICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShzZXJ2aWNlTmFtZSl9ICR7XG4gICAgICAgICAgICAgIGFnZW50LmFnZW50TmFtZVxuICAgICAgICAgICAgfWAsXG4gICAgICAgICAgICB0YXNrOiBhc3luYyAoKTogUHJvbWlzZTx2b2lkPiA9PiB7XG4gICAgICAgICAgICAgIGF3YWl0IGNvZGVnZW5SdXN0LmdlbmVyYXRlR2VuQWdlbnQoYWdlbnQpO1xuICAgICAgICAgICAgfSxcbiAgICAgICAgICB9KTtcbiAgICAgICAgfVxuICAgICAgfVxuXG4gICAgICB0YXNrcy5wdXNoKHtcbiAgICAgICAgdGl0bGU6IGBSdXN0IGZvcm1hdCAke3NlcnZpY2VOYW1lfWAsXG4gICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICBhd2FpdCBjb2RlZ2VuUnVzdC5mb3JtYXRDb2RlKCk7XG4gICAgICAgIH0sXG4gICAgICB9KTtcbiAgICB9XG4gIH1cblxuICByZXR1cm4gbmV3IExpc3RyKHRhc2tzLCB7IGNvbmN1cnJlbnQ6IGZhbHNlIH0pO1xufVxuIl19