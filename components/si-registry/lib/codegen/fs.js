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
    var pathname, basename, createdPath, codeFilename, codeOutput, rustfmtChild, exitPromise, _iteratorNormalCompletion, _didIteratorError, _iteratorError, _iterator, _step, _value, line, codeHash, existingCode, existingCodeHash;

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
              _context2.next = 58;
              break;
            }

            if (!codeFilename.endsWith(".rs")) {
              _context2.next = 51;
              break;
            }

            // @ts-ignore - we know what this is, right? ;0
            rustfmtChild = _child_process["default"].spawn("rustfmt", ["--emit", "stdout"], {
              stdio: ["pipe", "pipe", "pipe"]
            });
            exitPromise = (0, _stringio.onExit)(rustfmtChild);
            _context2.next = 13;
            return (0, _stringio.streamWrite)(rustfmtChild.stdin, code);

          case 13:
            _context2.next = 15;
            return (0, _stringio.streamEnd)(rustfmtChild.stdin);

          case 15:
            codeOutput = "";
            _iteratorNormalCompletion = true;
            _didIteratorError = false;
            _context2.prev = 18;
            _iterator = (0, _asyncIterator2["default"])((0, _stringio.chunksToLinesAsync)(rustfmtChild.stdout));

          case 20:
            _context2.next = 22;
            return _iterator.next();

          case 22:
            _step = _context2.sent;
            _iteratorNormalCompletion = _step.done;
            _context2.next = 26;
            return _step.value;

          case 26:
            _value = _context2.sent;

            if (_iteratorNormalCompletion) {
              _context2.next = 33;
              break;
            }

            line = _value;
            codeOutput += line;

          case 30:
            _iteratorNormalCompletion = true;
            _context2.next = 20;
            break;

          case 33:
            _context2.next = 39;
            break;

          case 35:
            _context2.prev = 35;
            _context2.t0 = _context2["catch"](18);
            _didIteratorError = true;
            _iteratorError = _context2.t0;

          case 39:
            _context2.prev = 39;
            _context2.prev = 40;

            if (!(!_iteratorNormalCompletion && _iterator["return"] != null)) {
              _context2.next = 44;
              break;
            }

            _context2.next = 44;
            return _iterator["return"]();

          case 44:
            _context2.prev = 44;

            if (!_didIteratorError) {
              _context2.next = 47;
              break;
            }

            throw _iteratorError;

          case 47:
            return _context2.finish(44);

          case 48:
            return _context2.finish(39);

          case 49:
            _context2.next = 51;
            return exitPromise;

          case 51:
            codeHash = _xxhash["default"].hash64(Buffer.from(codeOutput), 1234, "base64");
            _context2.next = 54;
            return _fs["default"].promises.readFile(codeFilename);

          case 54:
            existingCode = _context2.sent;
            existingCodeHash = _xxhash["default"].hash64(existingCode, 1234, "base64");

            if (!(codeHash == existingCodeHash)) {
              _context2.next = 58;
              break;
            }

            return _context2.abrupt("return");

          case 58:
            _context2.next = 60;
            return _fs["default"].promises.writeFile(codeFilename, codeOutput);

          case 60:
          case "end":
            return _context2.stop();
        }
      }
    }, _callee2, null, [[18, 35, 39, 49], [40,, 44, 48]]);
  }));
  return _writeCode.apply(this, arguments);
}
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL2ZzLnRzIl0sIm5hbWVzIjpbIm1ha2VQYXRoIiwicGF0aFBhcnQiLCJhYnNvbHV0ZVBhdGhOYW1lIiwicGF0aCIsInJlc29sdmUiLCJmcyIsImV4aXN0c1N5bmMiLCJwcm9taXNlcyIsIm1rZGlyIiwicmVjdXJzaXZlIiwid3JpdGVDb2RlIiwiZmlsZW5hbWUiLCJjb2RlIiwicGF0aG5hbWUiLCJkaXJuYW1lIiwiYmFzZW5hbWUiLCJjcmVhdGVkUGF0aCIsImNvZGVGaWxlbmFtZSIsImpvaW4iLCJjb2RlT3V0cHV0IiwiZW5kc1dpdGgiLCJydXN0Zm10Q2hpbGQiLCJjaGlsZFByb2Nlc3MiLCJzcGF3biIsInN0ZGlvIiwiZXhpdFByb21pc2UiLCJzdGRpbiIsInN0ZG91dCIsImxpbmUiLCJjb2RlSGFzaCIsIlhYSGFzaCIsImhhc2g2NCIsIkJ1ZmZlciIsImZyb20iLCJyZWFkRmlsZSIsImV4aXN0aW5nQ29kZSIsImV4aXN0aW5nQ29kZUhhc2giLCJ3cml0ZUZpbGUiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7QUFNQTs7QUFDQTs7QUFDQTs7QUFDQTs7U0FFc0JBLFE7Ozs7OzRGQUFmLGlCQUF3QkMsUUFBeEI7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQ0NDLFlBQUFBLGdCQURELEdBQ29CQyxpQkFBS0MsT0FBTCxDQUFhSCxRQUFiLENBRHBCOztBQUFBLGdCQUVBSSxlQUFHQyxVQUFILENBQWNKLGdCQUFkLENBRkE7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQSxtQkFHR0csZUFBR0UsUUFBSCxDQUFZQyxLQUFaLENBQWtCTixnQkFBbEIsRUFBb0M7QUFBRU8sY0FBQUEsU0FBUyxFQUFFO0FBQWIsYUFBcEMsQ0FISDs7QUFBQTtBQUFBLDZDQUtFUCxnQkFMRjs7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQSxHOzs7O1NBUWVRLFM7Ozs7OzZGQUFmLGtCQUF5QkMsUUFBekIsRUFBMkNDLElBQTNDO0FBQUE7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFDQ0MsWUFBQUEsUUFERCxHQUNZVixpQkFBS1csT0FBTCxDQUFhSCxRQUFiLENBRFo7QUFFQ0ksWUFBQUEsUUFGRCxHQUVZWixpQkFBS1ksUUFBTCxDQUFjSixRQUFkLENBRlo7QUFBQTtBQUFBLG1CQUdxQlgsUUFBUSxDQUFDYSxRQUFELENBSDdCOztBQUFBO0FBR0NHLFlBQUFBLFdBSEQ7QUFJQ0MsWUFBQUEsWUFKRCxHQUlnQmQsaUJBQUtlLElBQUwsQ0FBVUYsV0FBVixFQUF1QkQsUUFBdkIsQ0FKaEI7QUFLREksWUFBQUEsVUFMQyxHQUtZUCxJQUxaOztBQUFBLGlCQU1EUCxlQUFHQyxVQUFILENBQWNXLFlBQWQsQ0FOQztBQUFBO0FBQUE7QUFBQTs7QUFBQSxpQkFPQ0EsWUFBWSxDQUFDRyxRQUFiLENBQXNCLEtBQXRCLENBUEQ7QUFBQTtBQUFBO0FBQUE7O0FBUUQ7QUFDTUMsWUFBQUEsWUFUTCxHQVNvQkMsMEJBQWFDLEtBQWIsQ0FBbUIsU0FBbkIsRUFBOEIsQ0FBQyxRQUFELEVBQVcsUUFBWCxDQUE5QixFQUFvRDtBQUN2RUMsY0FBQUEsS0FBSyxFQUFFLENBQUMsTUFBRCxFQUFTLE1BQVQsRUFBaUIsTUFBakI7QUFEZ0UsYUFBcEQsQ0FUcEI7QUFZS0MsWUFBQUEsV0FaTCxHQVltQixzQkFBT0osWUFBUCxDQVpuQjtBQUFBO0FBQUEsbUJBYUssMkJBQVlBLFlBQVksQ0FBQ0ssS0FBekIsRUFBZ0NkLElBQWhDLENBYkw7O0FBQUE7QUFBQTtBQUFBLG1CQWNLLHlCQUFVUyxZQUFZLENBQUNLLEtBQXZCLENBZEw7O0FBQUE7QUFlRFAsWUFBQUEsVUFBVSxHQUFHLEVBQWI7QUFmQztBQUFBO0FBQUE7QUFBQSx3REFnQndCLGtDQUFtQkUsWUFBWSxDQUFDTSxNQUFoQyxDQWhCeEI7O0FBQUE7QUFBQTtBQUFBOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFnQmdCQyxZQUFBQSxJQWhCaEI7QUFpQkNULFlBQUFBLFVBQVUsSUFBSVMsSUFBZDs7QUFqQkQ7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBOztBQUFBO0FBQUE7QUFBQTtBQUFBOztBQUFBO0FBQUE7O0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFBQTs7QUFBQTtBQUFBOztBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBLG1CQW1CS0gsV0FuQkw7O0FBQUE7QUFxQkdJLFlBQUFBLFFBckJILEdBcUJjQyxtQkFBT0MsTUFBUCxDQUFjQyxNQUFNLENBQUNDLElBQVAsQ0FBWWQsVUFBWixDQUFkLEVBQXVDLElBQXZDLEVBQTZDLFFBQTdDLENBckJkO0FBQUE7QUFBQSxtQkFzQndCZCxlQUFHRSxRQUFILENBQVkyQixRQUFaLENBQXFCakIsWUFBckIsQ0F0QnhCOztBQUFBO0FBc0JHa0IsWUFBQUEsWUF0Qkg7QUF1QkdDLFlBQUFBLGdCQXZCSCxHQXVCc0JOLG1CQUFPQyxNQUFQLENBQWNJLFlBQWQsRUFBNEIsSUFBNUIsRUFBa0MsUUFBbEMsQ0F2QnRCOztBQUFBLGtCQXdCQ04sUUFBUSxJQUFJTyxnQkF4QmI7QUFBQTtBQUFBO0FBQUE7O0FBQUE7O0FBQUE7QUFBQTtBQUFBLG1CQTRCQy9CLGVBQUdFLFFBQUgsQ0FBWThCLFNBQVosQ0FBc0JwQixZQUF0QixFQUFvQ0UsVUFBcEMsQ0E1QkQ7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUEsRyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7XG4gIG9uRXhpdCxcbiAgY2h1bmtzVG9MaW5lc0FzeW5jLFxuICBzdHJlYW1Xcml0ZSxcbiAgc3RyZWFtRW5kLFxufSBmcm9tIFwiQHJhdXNjaG1hL3N0cmluZ2lvXCI7XG5pbXBvcnQgY2hpbGRQcm9jZXNzIGZyb20gXCJjaGlsZF9wcm9jZXNzXCI7XG5pbXBvcnQgZnMgZnJvbSBcImZzXCI7XG5pbXBvcnQgcGF0aCBmcm9tIFwicGF0aFwiO1xuaW1wb3J0IFhYSGFzaCBmcm9tIFwieHhoYXNoXCI7XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBtYWtlUGF0aChwYXRoUGFydDogc3RyaW5nKTogUHJvbWlzZTxzdHJpbmc+IHtcbiAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoUGFydCk7XG4gIGlmICghZnMuZXhpc3RzU3luYyhhYnNvbHV0ZVBhdGhOYW1lKSkge1xuICAgIGF3YWl0IGZzLnByb21pc2VzLm1rZGlyKGFic29sdXRlUGF0aE5hbWUsIHsgcmVjdXJzaXZlOiB0cnVlIH0pO1xuICB9XG4gIHJldHVybiBhYnNvbHV0ZVBhdGhOYW1lO1xufVxuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gd3JpdGVDb2RlKGZpbGVuYW1lOiBzdHJpbmcsIGNvZGU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuICBjb25zdCBwYXRobmFtZSA9IHBhdGguZGlybmFtZShmaWxlbmFtZSk7XG4gIGNvbnN0IGJhc2VuYW1lID0gcGF0aC5iYXNlbmFtZShmaWxlbmFtZSk7XG4gIGNvbnN0IGNyZWF0ZWRQYXRoID0gYXdhaXQgbWFrZVBhdGgocGF0aG5hbWUpO1xuICBjb25zdCBjb2RlRmlsZW5hbWUgPSBwYXRoLmpvaW4oY3JlYXRlZFBhdGgsIGJhc2VuYW1lKTtcbiAgbGV0IGNvZGVPdXRwdXQgPSBjb2RlO1xuICBpZiAoZnMuZXhpc3RzU3luYyhjb2RlRmlsZW5hbWUpKSB7XG4gICAgaWYgKGNvZGVGaWxlbmFtZS5lbmRzV2l0aChcIi5yc1wiKSkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGtub3cgd2hhdCB0aGlzIGlzLCByaWdodD8gOzBcbiAgICAgIGNvbnN0IHJ1c3RmbXRDaGlsZCA9IGNoaWxkUHJvY2Vzcy5zcGF3bihcInJ1c3RmbXRcIiwgW1wiLS1lbWl0XCIsIFwic3Rkb3V0XCJdLCB7XG4gICAgICAgIHN0ZGlvOiBbXCJwaXBlXCIsIFwicGlwZVwiLCBcInBpcGVcIl0sXG4gICAgICB9KTtcbiAgICAgIGNvbnN0IGV4aXRQcm9taXNlID0gb25FeGl0KHJ1c3RmbXRDaGlsZCk7XG4gICAgICBhd2FpdCBzdHJlYW1Xcml0ZShydXN0Zm10Q2hpbGQuc3RkaW4sIGNvZGUpO1xuICAgICAgYXdhaXQgc3RyZWFtRW5kKHJ1c3RmbXRDaGlsZC5zdGRpbik7XG4gICAgICBjb2RlT3V0cHV0ID0gXCJcIjtcbiAgICAgIGZvciBhd2FpdCAoY29uc3QgbGluZSBvZiBjaHVua3NUb0xpbmVzQXN5bmMocnVzdGZtdENoaWxkLnN0ZG91dCkpIHtcbiAgICAgICAgY29kZU91dHB1dCArPSBsaW5lO1xuICAgICAgfVxuICAgICAgYXdhaXQgZXhpdFByb21pc2U7XG4gICAgfVxuICAgIGNvbnN0IGNvZGVIYXNoID0gWFhIYXNoLmhhc2g2NChCdWZmZXIuZnJvbShjb2RlT3V0cHV0KSwgMTIzNCwgXCJiYXNlNjRcIik7XG4gICAgY29uc3QgZXhpc3RpbmdDb2RlID0gYXdhaXQgZnMucHJvbWlzZXMucmVhZEZpbGUoY29kZUZpbGVuYW1lKTtcbiAgICBjb25zdCBleGlzdGluZ0NvZGVIYXNoID0gWFhIYXNoLmhhc2g2NChleGlzdGluZ0NvZGUsIDEyMzQsIFwiYmFzZTY0XCIpO1xuICAgIGlmIChjb2RlSGFzaCA9PSBleGlzdGluZ0NvZGVIYXNoKSB7XG4gICAgICByZXR1cm47XG4gICAgfVxuICB9XG4gIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGVPdXRwdXQpO1xufVxuIl19