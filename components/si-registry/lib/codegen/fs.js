"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.makePath = makePath;
exports.writeCode = writeCode;

var _regenerator = _interopRequireDefault(require("@babel/runtime/regenerator"));

var _asyncToGenerator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncToGenerator"));

var _asyncIterator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncIterator"));

var _stringio = require("@rauschma/stringio");

var _child_process = _interopRequireDefault(require("child_process"));

var _fs = _interopRequireDefault(require("fs"));

var _path = _interopRequireDefault(require("path"));

var _xxhash = _interopRequireDefault(require("xxhash"));

function makePath(_x) {
  return _makePath.apply(this, arguments);
}

function _makePath() {
  _makePath = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee(pathPart) {
    var absolutePathName;
    return _regenerator["default"].wrap(function _callee$(_context) {
      while (1) {
        switch (_context.prev = _context.next) {
          case 0:
            absolutePathName = _path["default"].resolve(pathPart);

            if (_fs["default"].existsSync(absolutePathName)) {
              _context.next = 4;
              break;
            }

            _context.next = 4;
            return _fs["default"].promises.mkdir(absolutePathName, {
              recursive: true
            });

          case 4:
            return _context.abrupt("return", absolutePathName);

          case 5:
          case "end":
            return _context.stop();
        }
      }
    }, _callee);
  }));
  return _makePath.apply(this, arguments);
}

function writeCode(_x2, _x3) {
  return _writeCode.apply(this, arguments);
}

function _writeCode() {
  _writeCode = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee2(filename, code) {
    var pathname, basename, createdPath, codeFilename, codeOutput, rustfmtChild, _iteratorNormalCompletion, _didIteratorError, _iteratorError, _iterator, _step, _value, line, codeHash, existingCode, existingCodeHash;

    return _regenerator["default"].wrap(function _callee2$(_context2) {
      while (1) {
        switch (_context2.prev = _context2.next) {
          case 0:
            pathname = _path["default"].dirname(filename);
            basename = _path["default"].basename(filename);
            _context2.next = 4;
            return makePath(pathname);

          case 4:
            createdPath = _context2.sent;
            codeFilename = _path["default"].join(createdPath, basename);
            codeOutput = code;

            if (!_fs["default"].existsSync(codeFilename)) {
              _context2.next = 55;
              break;
            }

            if (!codeFilename.endsWith(".rs")) {
              _context2.next = 48;
              break;
            }

            // @ts-ignore - we know what this is, right? ;0
            rustfmtChild = _child_process["default"].spawn("rustfmt", ["--emit", "stdout"], {
              stdio: ["pipe", "pipe", "pipe"]
            });
            (0, _stringio.streamWrite)(rustfmtChild.stdin, code);
            (0, _stringio.streamEnd)(rustfmtChild.stdin);
            codeOutput = "";
            _iteratorNormalCompletion = true;
            _didIteratorError = false;
            _context2.prev = 15;
            _iterator = (0, _asyncIterator2["default"])((0, _stringio.chunksToLinesAsync)(rustfmtChild.stdout));

          case 17:
            _context2.next = 19;
            return _iterator.next();

          case 19:
            _step = _context2.sent;
            _iteratorNormalCompletion = _step.done;
            _context2.next = 23;
            return _step.value;

          case 23:
            _value = _context2.sent;

            if (_iteratorNormalCompletion) {
              _context2.next = 30;
              break;
            }

            line = _value;
            codeOutput += line;

          case 27:
            _iteratorNormalCompletion = true;
            _context2.next = 17;
            break;

          case 30:
            _context2.next = 36;
            break;

          case 32:
            _context2.prev = 32;
            _context2.t0 = _context2["catch"](15);
            _didIteratorError = true;
            _iteratorError = _context2.t0;

          case 36:
            _context2.prev = 36;
            _context2.prev = 37;

            if (!(!_iteratorNormalCompletion && _iterator["return"] != null)) {
              _context2.next = 41;
              break;
            }

            _context2.next = 41;
            return _iterator["return"]();

          case 41:
            _context2.prev = 41;

            if (!_didIteratorError) {
              _context2.next = 44;
              break;
            }

            throw _iteratorError;

          case 44:
            return _context2.finish(41);

          case 45:
            return _context2.finish(36);

          case 46:
            _context2.next = 48;
            return (0, _stringio.onExit)(rustfmtChild);

          case 48:
            codeHash = _xxhash["default"].hash64(Buffer.from(codeOutput), 1234, "base64");
            _context2.next = 51;
            return _fs["default"].promises.readFile(codeFilename);

          case 51:
            existingCode = _context2.sent;
            existingCodeHash = _xxhash["default"].hash64(existingCode, 1234, "base64");

            if (!(codeHash == existingCodeHash)) {
              _context2.next = 55;
              break;
            }

            return _context2.abrupt("return");

          case 55:
            _context2.next = 57;
            return _fs["default"].promises.writeFile(codeFilename, codeOutput);

          case 57:
          case "end":
            return _context2.stop();
        }
      }
    }, _callee2, null, [[15, 32, 36, 46], [37,, 41, 45]]);
  }));
  return _writeCode.apply(this, arguments);
}
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL2ZzLnRzIl0sIm5hbWVzIjpbIm1ha2VQYXRoIiwicGF0aFBhcnQiLCJhYnNvbHV0ZVBhdGhOYW1lIiwicGF0aCIsInJlc29sdmUiLCJmcyIsImV4aXN0c1N5bmMiLCJwcm9taXNlcyIsIm1rZGlyIiwicmVjdXJzaXZlIiwid3JpdGVDb2RlIiwiZmlsZW5hbWUiLCJjb2RlIiwicGF0aG5hbWUiLCJkaXJuYW1lIiwiYmFzZW5hbWUiLCJjcmVhdGVkUGF0aCIsImNvZGVGaWxlbmFtZSIsImpvaW4iLCJjb2RlT3V0cHV0IiwiZW5kc1dpdGgiLCJydXN0Zm10Q2hpbGQiLCJjaGlsZFByb2Nlc3MiLCJzcGF3biIsInN0ZGlvIiwic3RkaW4iLCJzdGRvdXQiLCJsaW5lIiwiY29kZUhhc2giLCJYWEhhc2giLCJoYXNoNjQiLCJCdWZmZXIiLCJmcm9tIiwicmVhZEZpbGUiLCJleGlzdGluZ0NvZGUiLCJleGlzdGluZ0NvZGVIYXNoIiwid3JpdGVGaWxlIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7O0FBTUE7O0FBQ0E7O0FBQ0E7O0FBQ0E7O1NBRXNCQSxROzs7Ozs0RkFBZixpQkFBd0JDLFFBQXhCO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUNDQyxZQUFBQSxnQkFERCxHQUNvQkMsaUJBQUtDLE9BQUwsQ0FBYUgsUUFBYixDQURwQjs7QUFBQSxnQkFFQUksZUFBR0MsVUFBSCxDQUFjSixnQkFBZCxDQUZBO0FBQUE7QUFBQTtBQUFBOztBQUFBO0FBQUEsbUJBR0dHLGVBQUdFLFFBQUgsQ0FBWUMsS0FBWixDQUFrQk4sZ0JBQWxCLEVBQW9DO0FBQUVPLGNBQUFBLFNBQVMsRUFBRTtBQUFiLGFBQXBDLENBSEg7O0FBQUE7QUFBQSw2Q0FLRVAsZ0JBTEY7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUEsRzs7OztTQVFlUSxTOzs7Ozs2RkFBZixrQkFBeUJDLFFBQXpCLEVBQTJDQyxJQUEzQztBQUFBOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQ0NDLFlBQUFBLFFBREQsR0FDWVYsaUJBQUtXLE9BQUwsQ0FBYUgsUUFBYixDQURaO0FBRUNJLFlBQUFBLFFBRkQsR0FFWVosaUJBQUtZLFFBQUwsQ0FBY0osUUFBZCxDQUZaO0FBQUE7QUFBQSxtQkFHcUJYLFFBQVEsQ0FBQ2EsUUFBRCxDQUg3Qjs7QUFBQTtBQUdDRyxZQUFBQSxXQUhEO0FBSUNDLFlBQUFBLFlBSkQsR0FJZ0JkLGlCQUFLZSxJQUFMLENBQVVGLFdBQVYsRUFBdUJELFFBQXZCLENBSmhCO0FBS0RJLFlBQUFBLFVBTEMsR0FLWVAsSUFMWjs7QUFBQSxpQkFNRFAsZUFBR0MsVUFBSCxDQUFjVyxZQUFkLENBTkM7QUFBQTtBQUFBO0FBQUE7O0FBQUEsaUJBT0NBLFlBQVksQ0FBQ0csUUFBYixDQUFzQixLQUF0QixDQVBEO0FBQUE7QUFBQTtBQUFBOztBQVFEO0FBQ01DLFlBQUFBLFlBVEwsR0FTb0JDLDBCQUFhQyxLQUFiLENBQW1CLFNBQW5CLEVBQThCLENBQUMsUUFBRCxFQUFXLFFBQVgsQ0FBOUIsRUFBb0Q7QUFDdkVDLGNBQUFBLEtBQUssRUFBRSxDQUFDLE1BQUQsRUFBUyxNQUFULEVBQWlCLE1BQWpCO0FBRGdFLGFBQXBELENBVHBCO0FBWUQsdUNBQVlILFlBQVksQ0FBQ0ksS0FBekIsRUFBZ0NiLElBQWhDO0FBQ0EscUNBQVVTLFlBQVksQ0FBQ0ksS0FBdkI7QUFDQU4sWUFBQUEsVUFBVSxHQUFHLEVBQWI7QUFkQztBQUFBO0FBQUE7QUFBQSx3REFld0Isa0NBQW1CRSxZQUFZLENBQUNLLE1BQWhDLENBZnhCOztBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBOztBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBZWdCQyxZQUFBQSxJQWZoQjtBQWdCQ1IsWUFBQUEsVUFBVSxJQUFJUSxJQUFkOztBQWhCRDtBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTs7QUFBQTtBQUFBOztBQUFBO0FBQUE7QUFBQTtBQUFBOztBQUFBOztBQUFBO0FBQUE7O0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBQUEsbUJBa0JLLHNCQUFPTixZQUFQLENBbEJMOztBQUFBO0FBb0JHTyxZQUFBQSxRQXBCSCxHQW9CY0MsbUJBQU9DLE1BQVAsQ0FBY0MsTUFBTSxDQUFDQyxJQUFQLENBQVliLFVBQVosQ0FBZCxFQUF1QyxJQUF2QyxFQUE2QyxRQUE3QyxDQXBCZDtBQUFBO0FBQUEsbUJBcUJ3QmQsZUFBR0UsUUFBSCxDQUFZMEIsUUFBWixDQUFxQmhCLFlBQXJCLENBckJ4Qjs7QUFBQTtBQXFCR2lCLFlBQUFBLFlBckJIO0FBc0JHQyxZQUFBQSxnQkF0QkgsR0FzQnNCTixtQkFBT0MsTUFBUCxDQUFjSSxZQUFkLEVBQTRCLElBQTVCLEVBQWtDLFFBQWxDLENBdEJ0Qjs7QUFBQSxrQkF1QkNOLFFBQVEsSUFBSU8sZ0JBdkJiO0FBQUE7QUFBQTtBQUFBOztBQUFBOztBQUFBO0FBQUE7QUFBQSxtQkEyQkM5QixlQUFHRSxRQUFILENBQVk2QixTQUFaLENBQXNCbkIsWUFBdEIsRUFBb0NFLFVBQXBDLENBM0JEOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLEciLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQge1xuICBvbkV4aXQsXG4gIGNodW5rc1RvTGluZXNBc3luYyxcbiAgc3RyZWFtV3JpdGUsXG4gIHN0cmVhbUVuZCxcbn0gZnJvbSBcIkByYXVzY2htYS9zdHJpbmdpb1wiO1xuaW1wb3J0IGNoaWxkUHJvY2VzcyBmcm9tIFwiY2hpbGRfcHJvY2Vzc1wiO1xuaW1wb3J0IGZzIGZyb20gXCJmc1wiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCBYWEhhc2ggZnJvbSBcInh4aGFzaFwiO1xuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gbWFrZVBhdGgocGF0aFBhcnQ6IHN0cmluZyk6IFByb21pc2U8c3RyaW5nPiB7XG4gIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aFBhcnQpO1xuICBpZiAoIWZzLmV4aXN0c1N5bmMoYWJzb2x1dGVQYXRoTmFtZSkpIHtcbiAgICBhd2FpdCBmcy5wcm9taXNlcy5ta2RpcihhYnNvbHV0ZVBhdGhOYW1lLCB7IHJlY3Vyc2l2ZTogdHJ1ZSB9KTtcbiAgfVxuICByZXR1cm4gYWJzb2x1dGVQYXRoTmFtZTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIHdyaXRlQ29kZShmaWxlbmFtZTogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbiAgY29uc3QgcGF0aG5hbWUgPSBwYXRoLmRpcm5hbWUoZmlsZW5hbWUpO1xuICBjb25zdCBiYXNlbmFtZSA9IHBhdGguYmFzZW5hbWUoZmlsZW5hbWUpO1xuICBjb25zdCBjcmVhdGVkUGF0aCA9IGF3YWl0IG1ha2VQYXRoKHBhdGhuYW1lKTtcbiAgY29uc3QgY29kZUZpbGVuYW1lID0gcGF0aC5qb2luKGNyZWF0ZWRQYXRoLCBiYXNlbmFtZSk7XG4gIGxldCBjb2RlT3V0cHV0ID0gY29kZTtcbiAgaWYgKGZzLmV4aXN0c1N5bmMoY29kZUZpbGVuYW1lKSkge1xuICAgIGlmIChjb2RlRmlsZW5hbWUuZW5kc1dpdGgoXCIucnNcIikpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBrbm93IHdoYXQgdGhpcyBpcywgcmlnaHQ/IDswXG4gICAgICBjb25zdCBydXN0Zm10Q2hpbGQgPSBjaGlsZFByb2Nlc3Muc3Bhd24oXCJydXN0Zm10XCIsIFtcIi0tZW1pdFwiLCBcInN0ZG91dFwiXSwge1xuICAgICAgICBzdGRpbzogW1wicGlwZVwiLCBcInBpcGVcIiwgXCJwaXBlXCJdLFxuICAgICAgfSk7XG4gICAgICBzdHJlYW1Xcml0ZShydXN0Zm10Q2hpbGQuc3RkaW4sIGNvZGUpO1xuICAgICAgc3RyZWFtRW5kKHJ1c3RmbXRDaGlsZC5zdGRpbik7XG4gICAgICBjb2RlT3V0cHV0ID0gXCJcIjtcbiAgICAgIGZvciBhd2FpdCAoY29uc3QgbGluZSBvZiBjaHVua3NUb0xpbmVzQXN5bmMocnVzdGZtdENoaWxkLnN0ZG91dCkpIHtcbiAgICAgICAgY29kZU91dHB1dCArPSBsaW5lO1xuICAgICAgfVxuICAgICAgYXdhaXQgb25FeGl0KHJ1c3RmbXRDaGlsZCk7XG4gICAgfVxuICAgIGNvbnN0IGNvZGVIYXNoID0gWFhIYXNoLmhhc2g2NChCdWZmZXIuZnJvbShjb2RlT3V0cHV0KSwgMTIzNCwgXCJiYXNlNjRcIik7XG4gICAgY29uc3QgZXhpc3RpbmdDb2RlID0gYXdhaXQgZnMucHJvbWlzZXMucmVhZEZpbGUoY29kZUZpbGVuYW1lKTtcbiAgICBjb25zdCBleGlzdGluZ0NvZGVIYXNoID0gWFhIYXNoLmhhc2g2NChleGlzdGluZ0NvZGUsIDEyMzQsIFwiYmFzZTY0XCIpO1xuICAgIGlmIChjb2RlSGFzaCA9PSBleGlzdGluZ0NvZGVIYXNoKSB7XG4gICAgICByZXR1cm47XG4gICAgfVxuICB9XG4gIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGVPdXRwdXQpO1xufVxuIl19