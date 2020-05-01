"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.ComponentAndEntityObject = exports.EntityEventObject = exports.EntityObject = exports.ComponentObject = exports.SystemObject = exports.BaseObject = void 0;

var _attrList = require("./attrList");

var _changeCase = require("change-case");

var _associations = require("./systemObject/associations");

function _typeof(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

function _inherits(subClass, superClass) { if (typeof superClass !== "function" && superClass !== null) { throw new TypeError("Super expression must either be null or a function"); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, writable: true, configurable: true } }); if (superClass) _setPrototypeOf(subClass, superClass); }

function _setPrototypeOf(o, p) { _setPrototypeOf = Object.setPrototypeOf || function _setPrototypeOf(o, p) { o.__proto__ = p; return o; }; return _setPrototypeOf(o, p); }

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = _getPrototypeOf(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = _getPrototypeOf(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return _possibleConstructorReturn(this, result); }; }

function _possibleConstructorReturn(self, call) { if (call && (_typeof(call) === "object" || typeof call === "function")) { return call; } return _assertThisInitialized(self); }

function _assertThisInitialized(self) { if (self === void 0) { throw new ReferenceError("this hasn't been initialised - super() hasn't been called"); } return self; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

function _getPrototypeOf(o) { _getPrototypeOf = Object.setPrototypeOf ? Object.getPrototypeOf : function _getPrototypeOf(o) { return o.__proto__ || Object.getPrototypeOf(o); }; return _getPrototypeOf(o); }

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

var BaseObject = /*#__PURE__*/function () {
  function BaseObject(_ref) {
    var typeName = _ref.typeName,
        displayTypeName = _ref.displayTypeName,
        serviceName = _ref.serviceName,
        _ref$siPathName = _ref.siPathName,
        siPathName = _ref$siPathName === void 0 ? "" : _ref$siPathName;

    _classCallCheck(this, BaseObject);

    _defineProperty(this, "typeName", void 0);

    _defineProperty(this, "displayTypeName", void 0);

    _defineProperty(this, "siPathName", void 0);

    _defineProperty(this, "serviceName", void 0);

    _defineProperty(this, "rootProp", void 0);

    _defineProperty(this, "methodsProp", void 0);

    _defineProperty(this, "associations", void 0);

    this.typeName = (0, _changeCase.camelCase)(typeName);
    this.displayTypeName = displayTypeName;
    this.siPathName = siPathName;
    this.serviceName = serviceName || typeName;
    this.rootProp = new _attrList.PropObject({
      name: typeName,
      label: displayTypeName,
      componentTypeName: typeName,
      parentName: ""
    });
    this.methodsProp = new _attrList.PropObject({
      name: "".concat(typeName),
      label: "".concat(displayTypeName, " Methods"),
      componentTypeName: typeName,
      parentName: ""
    });
    this.associations = new _associations.AssociationList();
  }

  _createClass(BaseObject, [{
    key: "kind",
    value: function kind() {
      return "baseObject";
    }
  }, {
    key: "fields",
    get: function get() {
      return this.rootProp.properties;
    }
  }, {
    key: "methods",
    get: function get() {
      return this.methodsProp.properties;
    }
  }]);

  return BaseObject;
}();

exports.BaseObject = BaseObject;

var SystemObject = /*#__PURE__*/function (_BaseObject) {
  _inherits(SystemObject, _BaseObject);

  var _super = _createSuper(SystemObject);

  function SystemObject(args) {
    var _this;

    _classCallCheck(this, SystemObject);

    _this = _super.call(this, args);

    _defineProperty(_assertThisInitialized(_this), "naturalKey", "name");

    _this.setSystemObjectDefaults();

    return _this;
  }

  _createClass(SystemObject, [{
    key: "setSystemObjectDefaults",
    value: function setSystemObjectDefaults() {
      this.fields.addText({
        name: "id",
        label: "".concat(this.displayTypeName, " ID"),
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
          p.required = true;
        }
      });
      this.fields.addText({
        name: "name",
        label: "".concat(this.displayTypeName, " Name"),
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
          p.required = true;
        }
      });
      this.fields.addText({
        name: "displayName",
        label: "".concat(this.displayTypeName, " Display Name"),
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
          p.required = true;
        }
      });
      this.fields.addLink({
        name: "siStorable",
        label: "SI Storable",
        options: function options(p) {
          p.universal = true;
          p.hidden = true;
          p.lookup = {
            typeName: "dataStorable"
          };
          p.required = true;
        }
      });
    }
  }, {
    key: "kind",
    value: function kind() {
      return "systemObject";
    }
  }, {
    key: "addGetMethod",
    value: function addGetMethod() {
      var args = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : {};
      // eslint-disable-next-line
      var systemObject = this;
      systemObject.methods.addMethod({
        name: "get",
        label: "Get a ".concat(systemObject.displayTypeName),
        options: function options(p) {
          p.isPrivate = args.isPrivate || false;
          p.request.properties.addText({
            name: "id",
            label: "".concat(systemObject.displayTypeName, " ID"),
            options: function options(p) {
              p.required = true;
            }
          });
          p.reply.properties.addLink({
            name: "object",
            label: "".concat(systemObject.displayTypeName, " Object"),
            options: function options(p) {
              p.lookup = {
                typeName: systemObject.typeName
              };
            }
          });
        }
      });
    }
  }, {
    key: "addListMethod",
    value: function addListMethod() {
      var args = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : {};
      // eslint-disable-next-line
      var systemObject = this;
      systemObject.methods.addMethod({
        name: "list",
        label: "List ".concat(systemObject.displayTypeName),
        options: function options(p) {
          p.universal = true;
          p.isPrivate = args.isPrivate || false;
          p.request.properties.addLink({
            name: "query",
            label: "Query",
            options: function options(p) {
              p.universal = true;
              p.lookup = {
                typeName: "dataQuery"
              };
            }
          });
          p.request.properties.addNumber({
            name: "pageSize",
            label: "Page Size",
            options: function options(p) {
              p.universal = true;
              p.numberKind = "uint32";
            }
          });
          p.request.properties.addText({
            name: "orderBy",
            label: "Order By",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.request.properties.addLink({
            name: "orderByDirection",
            label: "Order By Direction",
            options: function options(p) {
              p.universal = true;
              p.lookup = {
                typeName: "dataPageToken",
                names: ["orderByDirection"]
              };
            }
          });
          p.request.properties.addText({
            name: "pageToken",
            label: "Page Token",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.request.properties.addText({
            name: "scopeByTenantId",
            label: "Scope By Tenant ID",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.reply.properties.addLink({
            name: "items",
            label: "Items",
            options: function options(p) {
              p.universal = true;
              p.required = true;
              p.repeated = true;
              p.lookup = {
                typeName: systemObject.typeName
              };
            }
          });
          p.reply.properties.addNumber({
            name: "totalCount",
            label: "Total Count",
            options: function options(p) {
              p.universal = true;
              p.numberKind = "uint32";
            }
          });
          p.reply.properties.addText({
            name: "nextPageToken",
            label: "Next Page Token",
            options: function options(p) {
              p.universal = true;
            }
          });
        }
      });
    }
  }]);

  return SystemObject;
}(BaseObject);

exports.SystemObject = SystemObject;

var ComponentObject = /*#__PURE__*/function (_SystemObject) {
  _inherits(ComponentObject, _SystemObject);

  var _super2 = _createSuper(ComponentObject);

  function ComponentObject(args) {
    var _this2;

    _classCallCheck(this, ComponentObject);

    var typeName = "".concat(args.typeName, "Component");
    var displayTypeName = "".concat(args.displayTypeName, " Component");
    _this2 = _super2.call(this, {
      typeName: typeName,
      displayTypeName: displayTypeName,
      serviceName: args.serviceName
    });

    _defineProperty(_assertThisInitialized(_this2), "baseTypeName", void 0);

    _this2.baseTypeName = args.typeName;

    _this2.setComponentDefaults();

    return _this2;
  }

  _createClass(ComponentObject, [{
    key: "setComponentDefaults",
    value: function setComponentDefaults() {
      var baseTypeName = this.baseTypeName;
      this.fields.addText({
        name: "description",
        label: "Component Description",
        options: function options(p) {
          p.universal = true;
          p.required = true;
        }
      });
      this.fields.addText({
        name: "displayTypeName",
        label: "Display Type Name",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
          p.required = true;
        }
      });
      this.fields.addObject({
        name: "constraints",
        label: "Component Constraints",
        options: function options(p) {
          p.universal = true;
          p.required = true;
          p.properties.addText({
            name: "componentName",
            label: "Component Name",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.properties.addText({
            name: "componentDisplayName",
            label: "Component Display Name",
            options: function options(p) {
              p.universal = true;
            }
          });
        }
      });
      this.fields.addLink({
        name: "siProperties",
        label: "SI Properties",
        options: function options(p) {
          p.universal = true;
          p.lookup = {
            typeName: "componentSiProperties"
          };
          p.required = true;
        }
      });
      this.methods.addMethod({
        name: "get",
        label: "Get Component",
        options: function options(p) {
          p.required = true;
          p.request.properties.addText({
            name: "componentId",
            label: "".concat(this.displayTypeName, " ID"),
            options: function options(p) {
              p.universal = true;
              p.required = true;
            }
          });
          p.reply.properties.addLink({
            name: "component",
            label: this.displayTypeName,
            options: function options(p) {
              p.universal = true;
              p.required = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Component")
              };
            }
          });
        }
      });
      this.methods.addMethod({
        name: "list",
        label: "List Components",
        options: function options(p) {
          p.universal = true;
          p.request.properties.addLink({
            name: "query",
            label: "Query",
            options: function options(p) {
              p.universal = true;
              p.lookup = {
                typeName: "dataQuery"
              };
            }
          });
          p.request.properties.addNumber({
            name: "pageSize",
            label: "Page Size",
            options: function options(p) {
              p.universal = true;
              p.numberKind = "uint32";
            }
          });
          p.request.properties.addText({
            name: "orderBy",
            label: "Order By",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.request.properties.addLink({
            name: "orderByDirection",
            label: "Order By Direction",
            options: function options(p) {
              p.universal = true;
              p.lookup = {
                typeName: "dataPageToken",
                names: ["orderByDirection"]
              };
            }
          });
          p.request.properties.addText({
            name: "pageToken",
            label: "Page Token",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.request.properties.addText({
            name: "scopeByTenantId",
            label: "Scope By Tenant ID",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.reply.properties.addLink({
            name: "items",
            label: "Items",
            options: function options(p) {
              p.repeated = true;
              p.universal = true;
              p.required = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Component")
              };
            }
          });
          p.reply.properties.addNumber({
            name: "totalCount",
            label: "Total Count",
            options: function options(p) {
              p.universal = true;
              p.numberKind = "uint32";
            }
          });
          p.reply.properties.addText({
            name: "nextPageToken",
            label: "Next Page Token",
            options: function options(p) {
              p.universal = true;
            }
          });
        }
      });
      this.methods.addMethod({
        name: "pick",
        label: "Pick Component",
        options: function options(p) {
          p.request.properties.addLink({
            name: "constraints",
            label: "Constraints",
            options: function options(p) {
              p.universal = true;
              p.required = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Component"),
                names: ["constraints"]
              };
            }
          });
          p.reply.properties.addLink({
            name: "implicitConstraints",
            label: "Implicit Constraints",
            options: function options(p) {
              p.universal = true;
              p.required = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Component"),
                names: ["constraints"]
              };
            }
          });
          p.reply.properties.addLink({
            name: "component",
            label: "Chosen Component",
            options: function options(p) {
              p.universal = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Component")
              };
            }
          });
        }
      });
    }
  }, {
    key: "kind",
    value: function kind() {
      return "componentObject";
    }
  }, {
    key: "constraints",
    get: function get() {
      var constraintProp = this.fields.getEntry("constraints");
      return constraintProp.properties;
    }
  }]);

  return ComponentObject;
}(SystemObject);

exports.ComponentObject = ComponentObject;

var EntityObject = /*#__PURE__*/function (_SystemObject2) {
  _inherits(EntityObject, _SystemObject2);

  var _super3 = _createSuper(EntityObject);

  function EntityObject(args) {
    var _this3;

    _classCallCheck(this, EntityObject);

    var typeName = "".concat(args.typeName, "Entity");
    var displayTypeName = "".concat(args.displayTypeName, " Entity");
    _this3 = _super3.call(this, {
      typeName: typeName,
      displayTypeName: displayTypeName,
      serviceName: args.serviceName
    });

    _defineProperty(_assertThisInitialized(_this3), "baseTypeName", void 0);

    _this3.baseTypeName = args.typeName;

    _this3.setEntityDefaults();

    return _this3;
  }

  _createClass(EntityObject, [{
    key: "setEntityDefaults",
    value: function setEntityDefaults() {
      var baseTypeName = this.baseTypeName;
      this.fields.addText({
        name: "description",
        label: "Entity Description",
        options: function options(p) {
          p.universal = true;
          p.required = true;
        }
      });
      this.fields.addText({
        name: "displayTypeName",
        label: "Display Type Name",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
          p.required = true;
        }
      });
      this.fields.addLink({
        name: "siProperties",
        label: "SI Properties",
        options: function options(p) {
          p.universal = true;
          p.lookup = {
            typeName: "entitySiProperties"
          };
          p.required = true;
        }
      });
      this.fields.addObject({
        name: "properties",
        label: "Properties",
        options: function options(p) {
          p.universal = true;
          p.required = true;
        }
      });
      this.fields.addLink({
        name: "constraints",
        label: "Constraints",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
          p.lookup = {
            typeName: "".concat(baseTypeName, "Component"),
            names: ["constraints"]
          };
        }
      });
      this.fields.addLink({
        name: "implicitConstraints",
        label: "Implicit Constraints",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
          p.lookup = {
            typeName: "".concat(baseTypeName, "Component"),
            names: ["constraints"]
          };
        }
      });
      this.methods.addMethod({
        name: "create",
        label: "Create Entity",
        options: function options(p) {
          p.mutation = true;
          p.request.properties.addLink({
            name: "constraints",
            label: "Constraints",
            options: function options(p) {
              p.universal = true;
              p.readOnly = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Component"),
                names: ["constraints"]
              };
            }
          });
          p.request.properties.addLink({
            name: "properties",
            label: "Properties",
            options: function options(p) {
              p.universal = true;
              p.readOnly = true;
              p.required = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Entity"),
                names: ["properties"]
              };
            }
          });
          p.request.properties.addText({
            name: "name",
            label: "Name",
            options: function options(p) {
              p.required = true;
              p.universal = true;
            }
          });
          p.request.properties.addText({
            name: "displayName",
            label: "Display Name",
            options: function options(p) {
              p.required = true;
              p.universal = true;
            }
          });
          p.request.properties.addText({
            name: "description",
            label: "Description",
            options: function options(p) {
              p.required = true;
              p.universal = true;
            }
          });
          p.request.properties.addText({
            name: "workspaceId",
            label: "Workspace ID",
            options: function options(p) {
              p.universal = true;
              p.required = true;
            }
          });
          p.reply.properties.addLink({
            name: "entity",
            label: "Entity",
            options: function options(p) {
              p.universal = true;
              p.readOnly = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Entity")
              };
            }
          });
          p.reply.properties.addLink({
            name: "entityEvent",
            label: "Entity Event",
            options: function options(p) {
              p.universal = true;
              p.readOnly = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "EntityEvent")
              };
            }
          });
        }
      });
      this.methods.addMethod({
        name: "list",
        label: "List Entities",
        options: function options(p) {
          p.universal = true;
          p.request.properties.addLink({
            name: "query",
            label: "Query",
            options: function options(p) {
              p.universal = true;
              p.lookup = {
                typeName: "dataQuery"
              };
            }
          });
          p.request.properties.addNumber({
            name: "pageSize",
            label: "Page Size",
            options: function options(p) {
              p.universal = true;
              p.numberKind = "uint32";
            }
          });
          p.request.properties.addText({
            name: "orderBy",
            label: "Order By",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.request.properties.addLink({
            name: "orderByDirection",
            label: "Order By Direction",
            options: function options(p) {
              p.universal = true;
              p.lookup = {
                typeName: "dataPageToken",
                names: ["orderByDirection"]
              };
            }
          });
          p.request.properties.addText({
            name: "pageToken",
            label: "Page Token",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.request.properties.addText({
            name: "scopeByTenantId",
            label: "Scope By Tenant ID",
            options: function options(p) {
              p.universal = true;
            }
          });
          p.reply.properties.addLink({
            name: "items",
            label: "Items",
            options: function options(p) {
              p.repeated = true;
              p.universal = true;
              p.readOnly = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Entity")
              };
            }
          });
          p.reply.properties.addNumber({
            name: "totalCount",
            label: "Total Count",
            options: function options(p) {
              p.universal = true;
              p.numberKind = "uint32";
            }
          });
          p.reply.properties.addText({
            name: "nextPageToken",
            label: "Next Page Token",
            options: function options(p) {
              p.universal = true;
            }
          });
        }
      });
      this.methods.addMethod({
        name: "get",
        label: "Get Entity",
        options: function options(p) {
          p.universal = true;
          p.request.properties.addText({
            name: "entityId",
            label: "Entity ID",
            options: function options(p) {
              p.universal = true;
              p.required = true;
            }
          });
          p.reply.properties.addLink({
            name: "entity",
            label: "".concat(this.displayName),
            options: function options(p) {
              p.universal = true;
              p.readOnly = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Entity")
              };
            }
          });
        }
      });
      this.methods.addAction({
        name: "sync",
        label: "Sync State",
        options: function options(p) {
          p.mutation = true;
          p.universal = true;
        }
      });
    }
  }, {
    key: "kind",
    value: function kind() {
      return "entityObject";
    }
  }, {
    key: "properties",
    get: function get() {
      var prop = this.fields.getEntry("properties");
      return prop.properties;
    }
  }]);

  return EntityObject;
}(SystemObject);

exports.EntityObject = EntityObject;

var EntityEventObject = /*#__PURE__*/function (_SystemObject3) {
  _inherits(EntityEventObject, _SystemObject3);

  var _super4 = _createSuper(EntityEventObject);

  function EntityEventObject(args) {
    var _this4;

    _classCallCheck(this, EntityEventObject);

    var typeName = "".concat(args.typeName, "EntityEvent");
    var displayTypeName = "".concat(args.displayTypeName, " EntityEvent");
    _this4 = _super4.call(this, {
      typeName: typeName,
      displayTypeName: displayTypeName,
      serviceName: args.serviceName
    });

    _defineProperty(_assertThisInitialized(_this4), "baseTypeName", void 0);

    _this4.baseTypeName = args.typeName;

    _this4.setEntityEventDefaults();

    return _this4;
  }

  _createClass(EntityEventObject, [{
    key: "setEntityEventDefaults",
    value: function setEntityEventDefaults() {
      var baseTypeName = this.baseTypeName;
      this.fields.addText({
        name: "actionName",
        label: "Action Name",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
        }
      });
      this.fields.addText({
        name: "createTime",
        label: "Creation Time",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
        }
      });
      this.fields.addText({
        name: "updatedTime",
        label: "Updated Time",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
        }
      });
      this.fields.addText({
        name: "finalTime",
        label: "Final Time",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
        }
      });
      this.fields.addBool({
        name: "finalized",
        label: "Finalized",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
        }
      });
      this.fields.addBool({
        name: "success",
        label: "success",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
        }
      });
      this.fields.addText({
        name: "userId",
        label: "User ID",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
        }
      });
      this.fields.addLink({
        name: "siProperties",
        label: "SI Properties",
        options: function options(p) {
          p.universal = true;
          p.hidden = true;
          p.lookup = {
            typeName: "entityEventSiProperties"
          };
        }
      });
      this.fields.addLink({
        name: "inputEntity",
        label: "Input Entity",
        options: function options(p) {
          p.universal = true;
          p.hidden = true;
          p.lookup = {
            typeName: "".concat(baseTypeName, "Entity")
          };
        }
      });
      this.fields.addLink({
        name: "outputEntity",
        label: "Output Entity",
        options: function options(p) {
          p.universal = true;
          p.hidden = true;
          p.lookup = {
            typeName: "".concat(baseTypeName, "Entity")
          };
        }
      });
      this.fields.addLink({
        name: "previousEntity",
        label: "Previous Entity",
        options: function options(p) {
          p.universal = true;
          p.hidden = true;
          p.lookup = {
            typeName: "".concat(baseTypeName, "Entity")
          };
        }
      });
      this.fields.addText({
        name: "errorMessage",
        label: "Error Message",
        options: function options(p) {
          p.universal = true;
        }
      });
      this.fields.addText({
        name: "outputLines",
        label: "Output Lines",
        options: function options(p) {
          p.repeated = true;
          p.universal = true;
        }
      });
      this.fields.addText({
        name: "errorLines",
        label: "Error Lines",
        options: function options(p) {
          p.repeated = true;
          p.universal = true;
        }
      });
      this.addListMethod();
    }
  }, {
    key: "kind",
    value: function kind() {
      return "entityEventObject";
    }
  }]);

  return EntityEventObject;
}(SystemObject);

exports.EntityEventObject = EntityEventObject;

var ComponentAndEntityObject = /*#__PURE__*/function () {
  function ComponentAndEntityObject(args) {
    _classCallCheck(this, ComponentAndEntityObject);

    _defineProperty(this, "component", void 0);

    _defineProperty(this, "entity", void 0);

    _defineProperty(this, "entityEvent", void 0);

    this.component = new ComponentObject({
      typeName: args.typeName,
      displayTypeName: args.displayTypeName,
      siPathName: args.siPathName,
      serviceName: args.serviceName
    });
    this.entity = new EntityObject({
      typeName: args.typeName,
      displayTypeName: args.displayTypeName,
      siPathName: args.siPathName,
      serviceName: args.serviceName
    });
    this.entityEvent = new EntityEventObject({
      typeName: args.typeName,
      displayTypeName: args.displayTypeName,
      siPathName: args.siPathName,
      serviceName: args.serviceName
    });
  }

  _createClass(ComponentAndEntityObject, [{
    key: "properties",
    get: function get() {
      var prop = this.entity.fields.getEntry("properties");
      prop.properties.autoCreateEdits = true;
      return prop.properties;
    }
  }, {
    key: "constraints",
    get: function get() {
      var prop = this.component.fields.getEntry("constraints");
      return prop.properties;
    }
  }]);

  return ComponentAndEntityObject;
}();

exports.ComponentAndEntityObject = ComponentAndEntityObject;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9zeXN0ZW1Db21wb25lbnQudHMiXSwibmFtZXMiOlsiQmFzZU9iamVjdCIsInR5cGVOYW1lIiwiZGlzcGxheVR5cGVOYW1lIiwic2VydmljZU5hbWUiLCJzaVBhdGhOYW1lIiwicm9vdFByb3AiLCJQcm9wT2JqZWN0IiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJwYXJlbnROYW1lIiwibWV0aG9kc1Byb3AiLCJhc3NvY2lhdGlvbnMiLCJBc3NvY2lhdGlvbkxpc3QiLCJwcm9wZXJ0aWVzIiwiU3lzdGVtT2JqZWN0IiwiYXJncyIsInNldFN5c3RlbU9iamVjdERlZmF1bHRzIiwiZmllbGRzIiwiYWRkVGV4dCIsIm9wdGlvbnMiLCJwIiwidW5pdmVyc2FsIiwicmVhZE9ubHkiLCJyZXF1aXJlZCIsImFkZExpbmsiLCJoaWRkZW4iLCJsb29rdXAiLCJzeXN0ZW1PYmplY3QiLCJtZXRob2RzIiwiYWRkTWV0aG9kIiwiaXNQcml2YXRlIiwicmVxdWVzdCIsInJlcGx5IiwiYWRkTnVtYmVyIiwibnVtYmVyS2luZCIsIm5hbWVzIiwicmVwZWF0ZWQiLCJDb21wb25lbnRPYmplY3QiLCJiYXNlVHlwZU5hbWUiLCJzZXRDb21wb25lbnREZWZhdWx0cyIsImFkZE9iamVjdCIsImNvbnN0cmFpbnRQcm9wIiwiZ2V0RW50cnkiLCJFbnRpdHlPYmplY3QiLCJzZXRFbnRpdHlEZWZhdWx0cyIsIm11dGF0aW9uIiwiZGlzcGxheU5hbWUiLCJhZGRBY3Rpb24iLCJwcm9wIiwiRW50aXR5RXZlbnRPYmplY3QiLCJzZXRFbnRpdHlFdmVudERlZmF1bHRzIiwiYWRkQm9vbCIsImFkZExpc3RNZXRob2QiLCJDb21wb25lbnRBbmRFbnRpdHlPYmplY3QiLCJjb21wb25lbnQiLCJlbnRpdHkiLCJlbnRpdHlFdmVudCIsImF1dG9DcmVhdGVFZGl0cyJdLCJtYXBwaW5ncyI6Ijs7Ozs7OztBQUVBOztBQUNBOztBQUNBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7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Q1osVUFIa0I7QUFJbkNELE1BQUFBLFdBQVcsRUFBRWEsSUFBSSxDQUFDYjtBQUppQixLQUFwQixDQUFqQjtBQU1BLFNBQUtxRCxNQUFMLEdBQWMsSUFBSVosWUFBSixDQUFpQjtBQUM3QjNDLE1BQUFBLFFBQVEsRUFBRWUsSUFBSSxDQUFDZixRQURjO0FBRTdCQyxNQUFBQSxlQUFlLEVBQUVjLElBQUksQ0FBQ2QsZUFGTztBQUc3QkUsTUFBQUEsVUFBVSxFQUFFWSxJQUFJLENBQUNaLFVBSFk7QUFJN0JELE1BQUFBLFdBQVcsRUFBRWEsSUFBSSxDQUFDYjtBQUpXLEtBQWpCLENBQWQ7QUFNQSxTQUFLc0QsV0FBTCxHQUFtQixJQUFJUCxpQkFBSixDQUFzQjtBQUN2Q2pELE1BQUFBLFFBQVEsRUFBRWUsSUFBSSxDQUFDZixRQUR3QjtBQUV2Q0MsTUFBQUEsZUFBZSxFQUFFYyxJQUFJLENBQUNkLGVBRmlCO0FBR3ZDRSxNQUFBQSxVQUFVLEVBQUVZLElBQUksQ0FBQ1osVUFIc0I7QUFJdkNELE1BQUFBLFdBQVcsRUFBRWEsSUFBSSxDQUFDYjtBQUpxQixLQUF0QixDQUFuQjtBQU1EOzs7O3dCQUV3RDtBQUN2RCxVQUFNOEMsSUFBSSxHQUFHLEtBQUtPLE1BQUwsQ0FBWXRDLE1BQVosQ0FBbUJ5QixRQUFuQixDQUE0QixZQUE1QixDQUFiO0FBQ0FNLE1BQUFBLElBQUksQ0FBQ25DLFVBQUwsQ0FBZ0I0QyxlQUFoQixHQUFrQyxJQUFsQztBQUNBLGFBQU9ULElBQUksQ0FBQ25DLFVBQVo7QUFDRDs7O3dCQUU0RDtBQUMzRCxVQUFNbUMsSUFBSSxHQUFHLEtBQUtNLFNBQUwsQ0FBZXJDLE1BQWYsQ0FBc0J5QixRQUF0QixDQUErQixhQUEvQixDQUFiO0FBQ0EsYUFBT00sSUFBSSxDQUFDbkMsVUFBWjtBQUNEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcExpbmsgfSBmcm9tIFwiLi9wcm9wL2xpbmtcIjtcbmltcG9ydCB7IFByb3BOdW1iZXIgfSBmcm9tIFwiLi9wcm9wL251bWJlclwiO1xuaW1wb3J0IHsgUHJvcE9iamVjdCwgUHJvcE1ldGhvZCwgUHJvcEFjdGlvbiB9IGZyb20gXCIuL2F0dHJMaXN0XCI7XG5pbXBvcnQgeyBjYW1lbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCB7IEFzc29jaWF0aW9uTGlzdCB9IGZyb20gXCIuL3N5c3RlbU9iamVjdC9hc3NvY2lhdGlvbnNcIjtcblxuZXhwb3J0IHR5cGUgT2JqZWN0VHlwZXMgPVxuICB8IEJhc2VPYmplY3RcbiAgfCBTeXN0ZW1PYmplY3RcbiAgfCBDb21wb25lbnRPYmplY3RcbiAgfCBFbnRpdHlPYmplY3RcbiAgfCBFbnRpdHlFdmVudE9iamVjdDtcblxuZXhwb3J0IGludGVyZmFjZSBCYXNlT2JqZWN0Q29uc3RydWN0b3Ige1xuICB0eXBlTmFtZTogQmFzZU9iamVjdFtcInR5cGVOYW1lXCJdO1xuICBkaXNwbGF5VHlwZU5hbWU6IEJhc2VPYmplY3RbXCJkaXNwbGF5VHlwZU5hbWVcIl07XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHNpUGF0aE5hbWU/OiBzdHJpbmc7XG4gIG9wdGlvbnM/KGM6IEJhc2VPYmplY3QpOiB2b2lkO1xufVxuXG5leHBvcnQgaW50ZXJmYWNlIEFkZE1ldGhvZENvbnN0cnVjdG9yIHtcbiAgaXNQcml2YXRlPzogUHJvcE1ldGhvZFtcImlzUHJpdmF0ZVwiXTtcbn1cblxuZXhwb3J0IGNsYXNzIEJhc2VPYmplY3Qge1xuICB0eXBlTmFtZTogc3RyaW5nO1xuICBkaXNwbGF5VHlwZU5hbWU6IHN0cmluZztcbiAgc2lQYXRoTmFtZTogc3RyaW5nO1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuXG4gIHJvb3RQcm9wOiBQcm9wT2JqZWN0O1xuICBtZXRob2RzUHJvcDogUHJvcE9iamVjdDtcbiAgYXNzb2NpYXRpb25zOiBBc3NvY2lhdGlvbkxpc3Q7XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIHR5cGVOYW1lLFxuICAgIGRpc3BsYXlUeXBlTmFtZSxcbiAgICBzZXJ2aWNlTmFtZSxcbiAgICBzaVBhdGhOYW1lID0gXCJcIixcbiAgfTogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgdGhpcy50eXBlTmFtZSA9IGNhbWVsQ2FzZSh0eXBlTmFtZSk7XG4gICAgdGhpcy5kaXNwbGF5VHlwZU5hbWUgPSBkaXNwbGF5VHlwZU5hbWU7XG4gICAgdGhpcy5zaVBhdGhOYW1lID0gc2lQYXRoTmFtZTtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWUgfHwgdHlwZU5hbWU7XG4gICAgdGhpcy5yb290UHJvcCA9IG5ldyBQcm9wT2JqZWN0KHtcbiAgICAgIG5hbWU6IHR5cGVOYW1lLFxuICAgICAgbGFiZWw6IGRpc3BsYXlUeXBlTmFtZSxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0eXBlTmFtZSxcbiAgICAgIHBhcmVudE5hbWU6IFwiXCIsXG4gICAgfSk7XG4gICAgdGhpcy5tZXRob2RzUHJvcCA9IG5ldyBQcm9wT2JqZWN0KHtcbiAgICAgIG5hbWU6IGAke3R5cGVOYW1lfWAsXG4gICAgICBsYWJlbDogYCR7ZGlzcGxheVR5cGVOYW1lfSBNZXRob2RzYCxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0eXBlTmFtZSxcbiAgICAgIHBhcmVudE5hbWU6IFwiXCIsXG4gICAgfSk7XG4gICAgdGhpcy5hc3NvY2lhdGlvbnMgPSBuZXcgQXNzb2NpYXRpb25MaXN0KCk7XG4gIH1cblxuICBnZXQgZmllbGRzKCk6IEJhc2VPYmplY3RbXCJyb290UHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIHJldHVybiB0aGlzLnJvb3RQcm9wLnByb3BlcnRpZXM7XG4gIH1cblxuICBnZXQgbWV0aG9kcygpOiBCYXNlT2JqZWN0W1wibWV0aG9kc1Byb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICByZXR1cm4gdGhpcy5tZXRob2RzUHJvcC5wcm9wZXJ0aWVzO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImJhc2VPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgU3lzdGVtT2JqZWN0IGV4dGVuZHMgQmFzZU9iamVjdCB7XG4gIG5hdHVyYWxLZXkgPSBcIm5hbWVcIjtcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICBzdXBlcihhcmdzKTtcbiAgICB0aGlzLnNldFN5c3RlbU9iamVjdERlZmF1bHRzKCk7XG4gIH1cblxuICBzZXRTeXN0ZW1PYmplY3REZWZhdWx0cygpOiB2b2lkIHtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiaWRcIixcbiAgICAgIGxhYmVsOiBgJHt0aGlzLmRpc3BsYXlUeXBlTmFtZX0gSURgLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwibmFtZVwiLFxuICAgICAgbGFiZWw6IGAke3RoaXMuZGlzcGxheVR5cGVOYW1lfSBOYW1lYCxcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImRpc3BsYXlOYW1lXCIsXG4gICAgICBsYWJlbDogYCR7dGhpcy5kaXNwbGF5VHlwZU5hbWV9IERpc3BsYXkgTmFtZWAsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJzaVN0b3JhYmxlXCIsXG4gICAgICBsYWJlbDogXCJTSSBTdG9yYWJsZVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IFwiZGF0YVN0b3JhYmxlXCIsXG4gICAgICAgIH07XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJzeXN0ZW1PYmplY3RcIjtcbiAgfVxuXG4gIGFkZEdldE1ldGhvZChhcmdzOiBBZGRNZXRob2RDb25zdHJ1Y3RvciA9IHt9KTogdm9pZCB7XG4gICAgLy8gZXNsaW50LWRpc2FibGUtbmV4dC1saW5lXG4gICAgY29uc3Qgc3lzdGVtT2JqZWN0ID0gdGhpcztcbiAgICBzeXN0ZW1PYmplY3QubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJnZXRcIixcbiAgICAgIGxhYmVsOiBgR2V0IGEgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfWAsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5pc1ByaXZhdGUgPSBhcmdzLmlzUHJpdmF0ZSB8fCBmYWxzZTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJpZFwiLFxuICAgICAgICAgIGxhYmVsOiBgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfSBJRGAsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwib2JqZWN0XCIsXG4gICAgICAgICAgbGFiZWw6IGAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9IE9iamVjdGAsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBzeXN0ZW1PYmplY3QudHlwZU5hbWUsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxuXG4gIGFkZExpc3RNZXRob2QoYXJnczogQWRkTWV0aG9kQ29uc3RydWN0b3IgPSB7fSk6IHZvaWQge1xuICAgIC8vIGVzbGludC1kaXNhYmxlLW5leHQtbGluZVxuICAgIGNvbnN0IHN5c3RlbU9iamVjdCA9IHRoaXM7XG4gICAgc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYWRkTWV0aG9kKHtcbiAgICAgIG5hbWU6IFwibGlzdFwiLFxuICAgICAgbGFiZWw6IGBMaXN0ICR7c3lzdGVtT2JqZWN0LmRpc3BsYXlUeXBlTmFtZX1gLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTWV0aG9kKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5pc1ByaXZhdGUgPSBhcmdzLmlzUHJpdmF0ZSB8fCBmYWxzZTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJxdWVyeVwiLFxuICAgICAgICAgIGxhYmVsOiBcIlF1ZXJ5XCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBcImRhdGFRdWVyeVwiLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTnVtYmVyKHtcbiAgICAgICAgICBuYW1lOiBcInBhZ2VTaXplXCIsXG4gICAgICAgICAgbGFiZWw6IFwiUGFnZSBTaXplXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTnVtYmVyKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLm51bWJlcktpbmQgPSBcInVpbnQzMlwiO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcIm9yZGVyQnlcIixcbiAgICAgICAgICBsYWJlbDogXCJPcmRlciBCeVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcIm9yZGVyQnlEaXJlY3Rpb25cIixcbiAgICAgICAgICBsYWJlbDogXCJPcmRlciBCeSBEaXJlY3Rpb25cIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IFwiZGF0YVBhZ2VUb2tlblwiLFxuICAgICAgICAgICAgICBuYW1lczogW1wib3JkZXJCeURpcmVjdGlvblwiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwicGFnZVRva2VuXCIsXG4gICAgICAgICAgbGFiZWw6IFwiUGFnZSBUb2tlblwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcInNjb3BlQnlUZW5hbnRJZFwiLFxuICAgICAgICAgIGxhYmVsOiBcIlNjb3BlIEJ5IFRlbmFudCBJRFwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpdGVtc1wiLFxuICAgICAgICAgIGxhYmVsOiBcIkl0ZW1zXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcGVhdGVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogc3lzdGVtT2JqZWN0LnR5cGVOYW1lLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZE51bWJlcih7XG4gICAgICAgICAgbmFtZTogXCJ0b3RhbENvdW50XCIsXG4gICAgICAgICAgbGFiZWw6IFwiVG90YWwgQ291bnRcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BOdW1iZXIpIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubnVtYmVyS2luZCA9IFwidWludDMyXCI7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcIm5leHRQYWdlVG9rZW5cIixcbiAgICAgICAgICBsYWJlbDogXCJOZXh0IFBhZ2UgVG9rZW5cIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIENvbXBvbmVudE9iamVjdCBleHRlbmRzIFN5c3RlbU9iamVjdCB7XG4gIGJhc2VUeXBlTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIGNvbnN0IHR5cGVOYW1lID0gYCR7YXJncy50eXBlTmFtZX1Db21wb25lbnRgO1xuICAgIGNvbnN0IGRpc3BsYXlUeXBlTmFtZSA9IGAke2FyZ3MuZGlzcGxheVR5cGVOYW1lfSBDb21wb25lbnRgO1xuICAgIHN1cGVyKHtcbiAgICAgIHR5cGVOYW1lLFxuICAgICAgZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5iYXNlVHlwZU5hbWUgPSBhcmdzLnR5cGVOYW1lO1xuICAgIHRoaXMuc2V0Q29tcG9uZW50RGVmYXVsdHMoKTtcbiAgfVxuXG4gIHNldENvbXBvbmVudERlZmF1bHRzKCk6IHZvaWQge1xuICAgIGNvbnN0IGJhc2VUeXBlTmFtZSA9IHRoaXMuYmFzZVR5cGVOYW1lO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJkZXNjcmlwdGlvblwiLFxuICAgICAgbGFiZWw6IFwiQ29tcG9uZW50IERlc2NyaXB0aW9uXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImRpc3BsYXlUeXBlTmFtZVwiLFxuICAgICAgbGFiZWw6IFwiRGlzcGxheSBUeXBlIE5hbWVcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkT2JqZWN0KHtcbiAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgIGxhYmVsOiBcIkNvbXBvbmVudCBDb25zdHJhaW50c1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wT2JqZWN0KSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIHAucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImNvbXBvbmVudE5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJDb21wb25lbnQgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJjb21wb25lbnREaXNwbGF5TmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkNvbXBvbmVudCBEaXNwbGF5IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBcImNvbXBvbmVudFNpUHJvcGVydGllc1wiLFxuICAgICAgICB9O1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkTWV0aG9kKHtcbiAgICAgIG5hbWU6IFwiZ2V0XCIsXG4gICAgICBsYWJlbDogXCJHZXQgQ29tcG9uZW50XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiY29tcG9uZW50SWRcIixcbiAgICAgICAgICBsYWJlbDogYCR7dGhpcy5kaXNwbGF5VHlwZU5hbWV9IElEYCxcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJjb21wb25lbnRcIixcbiAgICAgICAgICBsYWJlbDogdGhpcy5kaXNwbGF5VHlwZU5hbWUsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkTWV0aG9kKHtcbiAgICAgIG5hbWU6IFwibGlzdFwiLFxuICAgICAgbGFiZWw6IFwiTGlzdCBDb21wb25lbnRzXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInF1ZXJ5XCIsXG4gICAgICAgICAgbGFiZWw6IFwiUXVlcnlcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IFwiZGF0YVF1ZXJ5XCIsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGROdW1iZXIoe1xuICAgICAgICAgIG5hbWU6IFwicGFnZVNpemVcIixcbiAgICAgICAgICBsYWJlbDogXCJQYWdlIFNpemVcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BOdW1iZXIpIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubnVtYmVyS2luZCA9IFwidWludDMyXCI7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwib3JkZXJCeVwiLFxuICAgICAgICAgIGxhYmVsOiBcIk9yZGVyIEJ5XCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwib3JkZXJCeURpcmVjdGlvblwiLFxuICAgICAgICAgIGxhYmVsOiBcIk9yZGVyIEJ5IERpcmVjdGlvblwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogXCJkYXRhUGFnZVRva2VuXCIsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJvcmRlckJ5RGlyZWN0aW9uXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJwYWdlVG9rZW5cIixcbiAgICAgICAgICBsYWJlbDogXCJQYWdlIFRva2VuXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwic2NvcGVCeVRlbmFudElkXCIsXG4gICAgICAgICAgbGFiZWw6IFwiU2NvcGUgQnkgVGVuYW50IElEXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcIml0ZW1zXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSXRlbXNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnJlcGVhdGVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGROdW1iZXIoe1xuICAgICAgICAgIG5hbWU6IFwidG90YWxDb3VudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIlRvdGFsIENvdW50XCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTnVtYmVyKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLm51bWJlcktpbmQgPSBcInVpbnQzMlwiO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJuZXh0UGFnZVRva2VuXCIsXG4gICAgICAgICAgbGFiZWw6IFwiTmV4dCBQYWdlIFRva2VuXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcInBpY2tcIixcbiAgICAgIGxhYmVsOiBcIlBpY2sgQ29tcG9uZW50XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJjb25zdHJhaW50c1wiLFxuICAgICAgICAgIGxhYmVsOiBcIkNvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpbXBsaWNpdENvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSW1wbGljaXQgQ29uc3RyYWludHNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImNvbXBvbmVudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIkNob3NlbiBDb21wb25lbnRcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cblxuICBnZXQgY29uc3RyYWludHMoKTogQ29tcG9uZW50T2JqZWN0W1wicm9vdFByb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICBjb25zdCBjb25zdHJhaW50UHJvcCA9IHRoaXMuZmllbGRzLmdldEVudHJ5KFwiY29uc3RyYWludHNcIikgYXMgUHJvcE9iamVjdDtcbiAgICByZXR1cm4gY29uc3RyYWludFByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJjb21wb25lbnRPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgRW50aXR5T2JqZWN0IGV4dGVuZHMgU3lzdGVtT2JqZWN0IHtcbiAgYmFzZVR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3IoYXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgY29uc3QgdHlwZU5hbWUgPSBgJHthcmdzLnR5cGVOYW1lfUVudGl0eWA7XG4gICAgY29uc3QgZGlzcGxheVR5cGVOYW1lID0gYCR7YXJncy5kaXNwbGF5VHlwZU5hbWV9IEVudGl0eWA7XG4gICAgc3VwZXIoe1xuICAgICAgdHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmJhc2VUeXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5zZXRFbnRpdHlEZWZhdWx0cygpO1xuICB9XG5cbiAgc2V0RW50aXR5RGVmYXVsdHMoKTogdm9pZCB7XG4gICAgY29uc3QgYmFzZVR5cGVOYW1lID0gdGhpcy5iYXNlVHlwZU5hbWU7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImRlc2NyaXB0aW9uXCIsXG4gICAgICBsYWJlbDogXCJFbnRpdHkgRGVzY3JpcHRpb25cIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiZGlzcGxheVR5cGVOYW1lXCIsXG4gICAgICBsYWJlbDogXCJEaXNwbGF5IFR5cGUgTmFtZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwic2lQcm9wZXJ0aWVzXCIsXG4gICAgICBsYWJlbDogXCJTSSBQcm9wZXJ0aWVzXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IFwiZW50aXR5U2lQcm9wZXJ0aWVzXCIsXG4gICAgICAgIH07XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRPYmplY3Qoe1xuICAgICAgbmFtZTogXCJwcm9wZXJ0aWVzXCIsXG4gICAgICBsYWJlbDogXCJQcm9wZXJ0aWVzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcImNvbnN0cmFpbnRzXCIsXG4gICAgICBsYWJlbDogXCJDb25zdHJhaW50c1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgbmFtZXM6IFtcImNvbnN0cmFpbnRzXCJdLFxuICAgICAgICB9O1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwiaW1wbGljaXRDb25zdHJhaW50c1wiLFxuICAgICAgbGFiZWw6IFwiSW1wbGljaXQgQ29uc3RyYWludHNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkTWV0aG9kKHtcbiAgICAgIG5hbWU6IFwiY3JlYXRlXCIsXG4gICAgICBsYWJlbDogXCJDcmVhdGUgRW50aXR5XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5tdXRhdGlvbiA9IHRydWU7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgICAgICBsYWJlbDogXCJDb25zdHJhaW50c1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgICAgbmFtZXM6IFtcImNvbnN0cmFpbnRzXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJwcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiUHJvcGVydGllc1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlgLFxuICAgICAgICAgICAgICBuYW1lczogW1wicHJvcGVydGllc1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwibmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIk5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImRpc3BsYXlOYW1lXCIsXG4gICAgICAgICAgbGFiZWw6IFwiRGlzcGxheSBOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJkZXNjcmlwdGlvblwiLFxuICAgICAgICAgIGxhYmVsOiBcIkRlc2NyaXB0aW9uXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJ3b3Jrc3BhY2VJZFwiLFxuICAgICAgICAgIGxhYmVsOiBcIldvcmtzcGFjZSBJRFwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImVudGl0eVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkVudGl0eVwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImVudGl0eUV2ZW50XCIsXG4gICAgICAgICAgbGFiZWw6IFwiRW50aXR5IEV2ZW50XCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlFdmVudGAsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcblxuICAgIHRoaXMubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJsaXN0XCIsXG4gICAgICBsYWJlbDogXCJMaXN0IEVudGl0aWVzXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInF1ZXJ5XCIsXG4gICAgICAgICAgbGFiZWw6IFwiUXVlcnlcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IFwiZGF0YVF1ZXJ5XCIsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGROdW1iZXIoe1xuICAgICAgICAgIG5hbWU6IFwicGFnZVNpemVcIixcbiAgICAgICAgICBsYWJlbDogXCJQYWdlIFNpemVcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BOdW1iZXIpIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubnVtYmVyS2luZCA9IFwidWludDMyXCI7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwib3JkZXJCeVwiLFxuICAgICAgICAgIGxhYmVsOiBcIk9yZGVyIEJ5XCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwib3JkZXJCeURpcmVjdGlvblwiLFxuICAgICAgICAgIGxhYmVsOiBcIk9yZGVyIEJ5IERpcmVjdGlvblwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogXCJkYXRhUGFnZVRva2VuXCIsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJvcmRlckJ5RGlyZWN0aW9uXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJwYWdlVG9rZW5cIixcbiAgICAgICAgICBsYWJlbDogXCJQYWdlIFRva2VuXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwic2NvcGVCeVRlbmFudElkXCIsXG4gICAgICAgICAgbGFiZWw6IFwiU2NvcGUgQnkgVGVuYW50IElEXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcIml0ZW1zXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSXRlbXNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnJlcGVhdGVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGROdW1iZXIoe1xuICAgICAgICAgIG5hbWU6IFwidG90YWxDb3VudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIlRvdGFsIENvdW50XCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTnVtYmVyKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLm51bWJlcktpbmQgPSBcInVpbnQzMlwiO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJuZXh0UGFnZVRva2VuXCIsXG4gICAgICAgICAgbGFiZWw6IFwiTmV4dCBQYWdlIFRva2VuXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImdldFwiLFxuICAgICAgbGFiZWw6IFwiR2V0IEVudGl0eVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTWV0aG9kKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJlbnRpdHlJZFwiLFxuICAgICAgICAgIGxhYmVsOiBcIkVudGl0eSBJRFwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImVudGl0eVwiLFxuICAgICAgICAgIGxhYmVsOiBgJHt0aGlzLmRpc3BsYXlOYW1lfWAsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkQWN0aW9uKHtcbiAgICAgIG5hbWU6IFwic3luY1wiLFxuICAgICAgbGFiZWw6IFwiU3luYyBTdGF0ZVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wQWN0aW9uKSB7XG4gICAgICAgIHAubXV0YXRpb24gPSB0cnVlO1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAgZ2V0IHByb3BlcnRpZXMoKTogRW50aXR5T2JqZWN0W1wicm9vdFByb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICBjb25zdCBwcm9wID0gdGhpcy5maWVsZHMuZ2V0RW50cnkoXCJwcm9wZXJ0aWVzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJlbnRpdHlPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgRW50aXR5RXZlbnRPYmplY3QgZXh0ZW5kcyBTeXN0ZW1PYmplY3Qge1xuICBiYXNlVHlwZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICBjb25zdCB0eXBlTmFtZSA9IGAke2FyZ3MudHlwZU5hbWV9RW50aXR5RXZlbnRgO1xuICAgIGNvbnN0IGRpc3BsYXlUeXBlTmFtZSA9IGAke2FyZ3MuZGlzcGxheVR5cGVOYW1lfSBFbnRpdHlFdmVudGA7XG4gICAgc3VwZXIoe1xuICAgICAgdHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmJhc2VUeXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5zZXRFbnRpdHlFdmVudERlZmF1bHRzKCk7XG4gIH1cblxuICBzZXRFbnRpdHlFdmVudERlZmF1bHRzKCk6IHZvaWQge1xuICAgIGNvbnN0IGJhc2VUeXBlTmFtZSA9IHRoaXMuYmFzZVR5cGVOYW1lO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJhY3Rpb25OYW1lXCIsXG4gICAgICBsYWJlbDogXCJBY3Rpb24gTmFtZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJjcmVhdGVUaW1lXCIsXG4gICAgICBsYWJlbDogXCJDcmVhdGlvbiBUaW1lXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcInVwZGF0ZWRUaW1lXCIsXG4gICAgICBsYWJlbDogXCJVcGRhdGVkIFRpbWVcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiZmluYWxUaW1lXCIsXG4gICAgICBsYWJlbDogXCJGaW5hbCBUaW1lXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkQm9vbCh7XG4gICAgICBuYW1lOiBcImZpbmFsaXplZFwiLFxuICAgICAgbGFiZWw6IFwiRmluYWxpemVkXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkQm9vbCh7XG4gICAgICBuYW1lOiBcInN1Y2Nlc3NcIixcbiAgICAgIGxhYmVsOiBcInN1Y2Nlc3NcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwidXNlcklkXCIsXG4gICAgICBsYWJlbDogXCJVc2VyIElEXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IFwiZW50aXR5RXZlbnRTaVByb3BlcnRpZXNcIixcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcImlucHV0RW50aXR5XCIsXG4gICAgICBsYWJlbDogXCJJbnB1dCBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcIm91dHB1dEVudGl0eVwiLFxuICAgICAgbGFiZWw6IFwiT3V0cHV0IEVudGl0eVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlgLFxuICAgICAgICB9O1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwicHJldmlvdXNFbnRpdHlcIixcbiAgICAgIGxhYmVsOiBcIlByZXZpb3VzIEVudGl0eVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlgLFxuICAgICAgICB9O1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiZXJyb3JNZXNzYWdlXCIsXG4gICAgICBsYWJlbDogXCJFcnJvciBNZXNzYWdlXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwib3V0cHV0TGluZXNcIixcbiAgICAgIGxhYmVsOiBcIk91dHB1dCBMaW5lc1wiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAucmVwZWF0ZWQgPSB0cnVlO1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJlcnJvckxpbmVzXCIsXG4gICAgICBsYWJlbDogXCJFcnJvciBMaW5lc1wiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAucmVwZWF0ZWQgPSB0cnVlO1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5hZGRMaXN0TWV0aG9kKCk7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiZW50aXR5RXZlbnRPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgaW50ZXJmYWNlIENvbXBvbmVudEFuZEVudGl0eU9iamVjdENvbnN0cnVjdG9yIHtcbiAgdHlwZU5hbWU6IEJhc2VPYmplY3RbXCJ0eXBlTmFtZVwiXTtcbiAgZGlzcGxheVR5cGVOYW1lOiBCYXNlT2JqZWN0W1wiZGlzcGxheVR5cGVOYW1lXCJdO1xuICBzaVBhdGhOYW1lPzogc3RyaW5nO1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBvcHRpb25zPyhjOiBDb21wb25lbnRBbmRFbnRpdHlPYmplY3QpOiB2b2lkO1xufVxuXG5leHBvcnQgY2xhc3MgQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IHtcbiAgY29tcG9uZW50OiBDb21wb25lbnRPYmplY3Q7XG4gIGVudGl0eTogRW50aXR5T2JqZWN0O1xuICBlbnRpdHlFdmVudDogRW50aXR5RXZlbnRPYmplY3Q7XG5cbiAgY29uc3RydWN0b3IoYXJnczogQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICB0aGlzLmNvbXBvbmVudCA9IG5ldyBDb21wb25lbnRPYmplY3Qoe1xuICAgICAgdHlwZU5hbWU6IGFyZ3MudHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWU6IGFyZ3MuZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2lQYXRoTmFtZTogYXJncy5zaVBhdGhOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5lbnRpdHkgPSBuZXcgRW50aXR5T2JqZWN0KHtcbiAgICAgIHR5cGVOYW1lOiBhcmdzLnR5cGVOYW1lLFxuICAgICAgZGlzcGxheVR5cGVOYW1lOiBhcmdzLmRpc3BsYXlUeXBlTmFtZSxcbiAgICAgIHNpUGF0aE5hbWU6IGFyZ3Muc2lQYXRoTmFtZSxcbiAgICAgIHNlcnZpY2VOYW1lOiBhcmdzLnNlcnZpY2VOYW1lLFxuICAgIH0pO1xuICAgIHRoaXMuZW50aXR5RXZlbnQgPSBuZXcgRW50aXR5RXZlbnRPYmplY3Qoe1xuICAgICAgdHlwZU5hbWU6IGFyZ3MudHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWU6IGFyZ3MuZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2lQYXRoTmFtZTogYXJncy5zaVBhdGhOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gIH1cblxuICBnZXQgcHJvcGVydGllcygpOiBFbnRpdHlPYmplY3RbXCJyb290UHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIGNvbnN0IHByb3AgPSB0aGlzLmVudGl0eS5maWVsZHMuZ2V0RW50cnkoXCJwcm9wZXJ0aWVzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcHJvcC5wcm9wZXJ0aWVzLmF1dG9DcmVhdGVFZGl0cyA9IHRydWU7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGdldCBjb25zdHJhaW50cygpOiBDb21wb25lbnRPYmplY3RbXCJyb290UHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIGNvbnN0IHByb3AgPSB0aGlzLmNvbXBvbmVudC5maWVsZHMuZ2V0RW50cnkoXCJjb25zdHJhaW50c1wiKSBhcyBQcm9wT2JqZWN0O1xuICAgIHJldHVybiBwcm9wLnByb3BlcnRpZXM7XG4gIH1cbn1cbiJdfQ==