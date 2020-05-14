"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.ComponentAndEntityObject = exports.EntityEventObject = exports.EntityObject = exports.ComponentObject = exports.SystemObject = exports.BaseObject = void 0;

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _attrList = require("./attrList");

var _changeCase = require("change-case");

var _associations = require("./systemObject/associations");

var _graphql = require("./systemObject/graphql");

function _createSuper(Derived) { return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (_isNativeReflectConstruct()) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var BaseObject = /*#__PURE__*/function () {
  function BaseObject(_ref) {
    var typeName = _ref.typeName,
        displayTypeName = _ref.displayTypeName,
        serviceName = _ref.serviceName,
        _ref$siPathName = _ref.siPathName,
        siPathName = _ref$siPathName === void 0 ? "" : _ref$siPathName;
    (0, _classCallCheck2["default"])(this, BaseObject);
    (0, _defineProperty2["default"])(this, "typeName", void 0);
    (0, _defineProperty2["default"])(this, "displayTypeName", void 0);
    (0, _defineProperty2["default"])(this, "siPathName", void 0);
    (0, _defineProperty2["default"])(this, "serviceName", void 0);
    (0, _defineProperty2["default"])(this, "mvcc", void 0);
    (0, _defineProperty2["default"])(this, "rootProp", void 0);
    (0, _defineProperty2["default"])(this, "methodsProp", void 0);
    (0, _defineProperty2["default"])(this, "associations", void 0);
    (0, _defineProperty2["default"])(this, "internalGraphql", void 0);
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
    this.internalGraphql = undefined;
    this.mvcc = false;
  }

  (0, _createClass2["default"])(BaseObject, [{
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
  }, {
    key: "graphql",
    get: function get() {
      if (this.internalGraphql == undefined) {
        this.internalGraphql = new _graphql.SiGraphql(this);
      }

      return this.internalGraphql;
    }
  }]);
  return BaseObject;
}();

exports.BaseObject = BaseObject;

var SystemObject = /*#__PURE__*/function (_BaseObject) {
  (0, _inherits2["default"])(SystemObject, _BaseObject);

  var _super = _createSuper(SystemObject);

  function SystemObject(args) {
    var _this;

    (0, _classCallCheck2["default"])(this, SystemObject);
    _this = _super.call(this, args);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "naturalKey", "name");
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "migrateable", false);

    _this.setSystemObjectDefaults();

    return _this;
  }

  (0, _createClass2["default"])(SystemObject, [{
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
            name: "item",
            label: "".concat(systemObject.displayTypeName, " Item"),
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
  (0, _inherits2["default"])(ComponentObject, _SystemObject);

  var _super2 = _createSuper(ComponentObject);

  function ComponentObject(args) {
    var _this2;

    (0, _classCallCheck2["default"])(this, ComponentObject);
    var typeName = "".concat(args.typeName, "Component");
    var displayTypeName = "".concat(args.displayTypeName, " Component");
    _this2 = _super2.call(this, {
      typeName: typeName,
      displayTypeName: displayTypeName,
      serviceName: args.serviceName
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "baseTypeName", void 0);
    _this2.baseTypeName = args.typeName;

    _this2.setComponentDefaults();

    return _this2;
  }

  (0, _createClass2["default"])(ComponentObject, [{
    key: "setComponentDefaults",
    value: function setComponentDefaults() {
      var baseTypeName = this.baseTypeName;
      this.migrateable = true;
      this.addGetMethod();
      this.addListMethod();
      this.fields.addText({
        name: "description",
        label: "Component Description",
        options: function options(p) {
          p.universal = true;
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
        name: "create",
        label: "Create a Component",
        options: function options(p) {
          p.mutation = true;
          p.hidden = true;
          p.isPrivate = true;
          p.request.properties.addText({
            name: "name",
            label: "Integration Name",
            options: function options(p) {
              p.required = true;
            }
          });
          p.request.properties.addText({
            name: "displayName",
            label: "Integration Display Name",
            options: function options(p) {
              p.required = true;
            }
          });
          p.request.properties.addText({
            name: "description",
            label: "Integration Display Name",
            options: function options(p) {
              p.required = true;
            }
          });
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
          p.request.properties.addLink({
            name: "siProperties",
            label: "Si Properties",
            options: function options(p) {
              p.required = true;
              p.lookup = {
                typeName: "componentSiProperties"
              };
            }
          });
          p.reply.properties.addLink({
            name: "item",
            label: "".concat(baseTypeName, "Component Item"),
            options: function options(p) {
              p.universal = true;
              p.readOnly = true;
              p.lookup = {
                typeName: "".concat(baseTypeName, "Component")
              };
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
  (0, _inherits2["default"])(EntityObject, _SystemObject2);

  var _super3 = _createSuper(EntityObject);

  function EntityObject(args) {
    var _this3;

    (0, _classCallCheck2["default"])(this, EntityObject);
    var typeName = "".concat(args.typeName, "Entity");
    var displayTypeName = "".concat(args.displayTypeName, " Entity");
    _this3 = _super3.call(this, {
      typeName: typeName,
      displayTypeName: displayTypeName,
      serviceName: args.serviceName
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this3), "baseTypeName", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this3), "integrationServices", void 0);
    _this3.baseTypeName = args.typeName;
    _this3.integrationServices = [];

    _this3.setEntityDefaults();

    return _this3;
  }

  (0, _createClass2["default"])(EntityObject, [{
    key: "setEntityDefaults",
    value: function setEntityDefaults() {
      var baseTypeName = this.baseTypeName;
      this.mvcc = true;
      this.addGetMethod();
      this.addListMethod();
      this.fields.addText({
        name: "description",
        label: "Entity Description",
        options: function options(p) {
          p.universal = true;
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
            name: "workspace_id",
            label: "Workspace ID",
            options: function options(p) {
              p.required = true;
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
          p.reply.properties.addLink({
            name: "item",
            label: "${baseTypeName}Entity Item",
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
  (0, _inherits2["default"])(EntityEventObject, _SystemObject3);

  var _super4 = _createSuper(EntityEventObject);

  function EntityEventObject(args) {
    var _this4;

    (0, _classCallCheck2["default"])(this, EntityEventObject);
    var typeName = "".concat(args.typeName, "EntityEvent");
    var displayTypeName = "".concat(args.displayTypeName, " EntityEvent");
    _this4 = _super4.call(this, {
      typeName: typeName,
      displayTypeName: displayTypeName,
      serviceName: args.serviceName
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this4), "baseTypeName", void 0);
    _this4.baseTypeName = args.typeName;

    _this4.setEntityEventDefaults();

    return _this4;
  }

  (0, _createClass2["default"])(EntityEventObject, [{
    key: "setEntityEventDefaults",
    value: function setEntityEventDefaults() {
      var baseTypeName = this.baseTypeName;
      this.fields.addText({
        name: "actionName",
        label: "Action Name",
        options: function options(p) {
          p.universal = true;
          p.required = true;
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
        name: "success",
        label: "success",
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
      this.fields.addText({
        name: "userId",
        label: "User ID",
        options: function options(p) {
          p.universal = true;
          p.readOnly = true;
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
      this.fields.addText({
        name: "errorMessage",
        label: "Error Message",
        options: function options(p) {
          p.universal = true;
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
      this.fields.addLink({
        name: "inputEntity",
        label: "Input Entity",
        options: function options(p) {
          p.universal = true;
          p.required = true;
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
    (0, _classCallCheck2["default"])(this, ComponentAndEntityObject);
    (0, _defineProperty2["default"])(this, "component", void 0);
    (0, _defineProperty2["default"])(this, "entity", void 0);
    (0, _defineProperty2["default"])(this, "entityEvent", void 0);
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

  (0, _createClass2["default"])(ComponentAndEntityObject, [{
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9zeXN0ZW1Db21wb25lbnQudHMiXSwibmFtZXMiOlsiQmFzZU9iamVjdCIsInR5cGVOYW1lIiwiZGlzcGxheVR5cGVOYW1lIiwic2VydmljZU5hbWUiLCJzaVBhdGhOYW1lIiwicm9vdFByb3AiLCJQcm9wT2JqZWN0IiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJwYXJlbnROYW1lIiwibWV0aG9kc1Byb3AiLCJhc3NvY2lhdGlvbnMiLCJBc3NvY2lhdGlvbkxpc3QiLCJpbnRlcm5hbEdyYXBocWwiLCJ1bmRlZmluZWQiLCJtdmNjIiwicHJvcGVydGllcyIsIlNpR3JhcGhxbCIsIlN5c3RlbU9iamVjdCIsImFyZ3MiLCJzZXRTeXN0ZW1PYmplY3REZWZhdWx0cyIsImZpZWxkcyIsImFkZFRleHQiLCJvcHRpb25zIiwicCIsInVuaXZlcnNhbCIsInJlYWRPbmx5IiwicmVxdWlyZWQiLCJhZGRMaW5rIiwiaGlkZGVuIiwibG9va3VwIiwic3lzdGVtT2JqZWN0IiwibWV0aG9kcyIsImFkZE1ldGhvZCIsImlzUHJpdmF0ZSIsInJlcXVlc3QiLCJyZXBseSIsImFkZE51bWJlciIsIm51bWJlcktpbmQiLCJuYW1lcyIsInJlcGVhdGVkIiwiQ29tcG9uZW50T2JqZWN0IiwiYmFzZVR5cGVOYW1lIiwic2V0Q29tcG9uZW50RGVmYXVsdHMiLCJtaWdyYXRlYWJsZSIsImFkZEdldE1ldGhvZCIsImFkZExpc3RNZXRob2QiLCJhZGRPYmplY3QiLCJtdXRhdGlvbiIsImNvbnN0cmFpbnRQcm9wIiwiZ2V0RW50cnkiLCJFbnRpdHlPYmplY3QiLCJpbnRlZ3JhdGlvblNlcnZpY2VzIiwic2V0RW50aXR5RGVmYXVsdHMiLCJhZGRBY3Rpb24iLCJwcm9wIiwiRW50aXR5RXZlbnRPYmplY3QiLCJzZXRFbnRpdHlFdmVudERlZmF1bHRzIiwiYWRkQm9vbCIsIkNvbXBvbmVudEFuZEVudGl0eU9iamVjdCIsImNvbXBvbmVudCIsImVudGl0eSIsImVu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Qm5ELE1BQUFBLFFBQVEsRUFBRW1CLElBQUksQ0FBQ25CLFFBRGM7QUFFN0JDLE1BQUFBLGVBQWUsRUFBRWtCLElBQUksQ0FBQ2xCLGVBRk87QUFHN0JFLE1BQUFBLFVBQVUsRUFBRWdCLElBQUksQ0FBQ2hCLFVBSFk7QUFJN0JELE1BQUFBLFdBQVcsRUFBRWlCLElBQUksQ0FBQ2pCO0FBSlcsS0FBakIsQ0FBZDtBQU1BLFNBQUs0RCxXQUFMLEdBQW1CLElBQUlOLGlCQUFKLENBQXNCO0FBQ3ZDeEQsTUFBQUEsUUFBUSxFQUFFbUIsSUFBSSxDQUFDbkIsUUFEd0I7QUFFdkNDLE1BQUFBLGVBQWUsRUFBRWtCLElBQUksQ0FBQ2xCLGVBRmlCO0FBR3ZDRSxNQUFBQSxVQUFVLEVBQUVnQixJQUFJLENBQUNoQixVQUhzQjtBQUl2Q0QsTUFBQUEsV0FBVyxFQUFFaUIsSUFBSSxDQUFDakI7QUFKcUIsS0FBdEIsQ0FBbkI7QUFNRDs7Ozt3QkFFd0Q7QUFDdkQsVUFBTXFELElBQUksR0FBRyxLQUFLTSxNQUFMLENBQVl4QyxNQUFaLENBQW1CNkIsUUFBbkIsQ0FBNEIsWUFBNUIsQ0FBYjtBQUNBSyxNQUFBQSxJQUFJLENBQUN2QyxVQUFMLENBQWdCK0MsZUFBaEIsR0FBa0MsSUFBbEM7QUFDQSxhQUFPUixJQUFJLENBQUN2QyxVQUFaO0FBQ0Q7Ozt3QkFFNEQ7QUFDM0QsVUFBTXVDLElBQUksR0FBRyxLQUFLSyxTQUFMLENBQWV2QyxNQUFmLENBQXNCNkIsUUFBdEIsQ0FBK0IsYUFBL0IsQ0FBYjtBQUNBLGFBQU9LLElBQUksQ0FBQ3ZDLFVBQVo7QUFDRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3BMaW5rIH0gZnJvbSBcIi4vcHJvcC9saW5rXCI7XG5pbXBvcnQgeyBQcm9wTnVtYmVyIH0gZnJvbSBcIi4vcHJvcC9udW1iZXJcIjtcbmltcG9ydCB7XG4gIFByb3BPYmplY3QsXG4gIFByb3BNZXRob2QsXG4gIFByb3BBY3Rpb24sXG4gIEludGVncmF0aW9uU2VydmljZSxcbn0gZnJvbSBcIi4vYXR0ckxpc3RcIjtcbmltcG9ydCB7IGNhbWVsQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuaW1wb3J0IHsgQXNzb2NpYXRpb25MaXN0IH0gZnJvbSBcIi4vc3lzdGVtT2JqZWN0L2Fzc29jaWF0aW9uc1wiO1xuaW1wb3J0IHsgU2lHcmFwaHFsIH0gZnJvbSBcIi4vc3lzdGVtT2JqZWN0L2dyYXBocWxcIjtcblxuZXhwb3J0IHR5cGUgT2JqZWN0VHlwZXMgPVxuICB8IEJhc2VPYmplY3RcbiAgfCBTeXN0ZW1PYmplY3RcbiAgfCBDb21wb25lbnRPYmplY3RcbiAgfCBFbnRpdHlPYmplY3RcbiAgfCBFbnRpdHlFdmVudE9iamVjdDtcblxuZXhwb3J0IGludGVyZmFjZSBCYXNlT2JqZWN0Q29uc3RydWN0b3Ige1xuICB0eXBlTmFtZTogQmFzZU9iamVjdFtcInR5cGVOYW1lXCJdO1xuICBkaXNwbGF5VHlwZU5hbWU6IEJhc2VPYmplY3RbXCJkaXNwbGF5VHlwZU5hbWVcIl07XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHNpUGF0aE5hbWU/OiBzdHJpbmc7XG4gIG9wdGlvbnM/KGM6IEJhc2VPYmplY3QpOiB2b2lkO1xufVxuXG5leHBvcnQgaW50ZXJmYWNlIEFkZE1ldGhvZENvbnN0cnVjdG9yIHtcbiAgaXNQcml2YXRlPzogUHJvcE1ldGhvZFtcImlzUHJpdmF0ZVwiXTtcbn1cblxuZXhwb3J0IGNsYXNzIEJhc2VPYmplY3Qge1xuICB0eXBlTmFtZTogc3RyaW5nO1xuICBkaXNwbGF5VHlwZU5hbWU6IHN0cmluZztcbiAgc2lQYXRoTmFtZTogc3RyaW5nO1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBtdmNjOiBib29sZWFuO1xuXG4gIHJvb3RQcm9wOiBQcm9wT2JqZWN0O1xuICBtZXRob2RzUHJvcDogUHJvcE9iamVjdDtcbiAgYXNzb2NpYXRpb25zOiBBc3NvY2lhdGlvbkxpc3Q7XG5cbiAgcHJpdmF0ZSBpbnRlcm5hbEdyYXBocWw6IHVuZGVmaW5lZCB8IFNpR3JhcGhxbDtcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgdHlwZU5hbWUsXG4gICAgZGlzcGxheVR5cGVOYW1lLFxuICAgIHNlcnZpY2VOYW1lLFxuICAgIHNpUGF0aE5hbWUgPSBcIlwiLFxuICB9OiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICB0aGlzLnR5cGVOYW1lID0gY2FtZWxDYXNlKHR5cGVOYW1lKTtcbiAgICB0aGlzLmRpc3BsYXlUeXBlTmFtZSA9IGRpc3BsYXlUeXBlTmFtZTtcbiAgICB0aGlzLnNpUGF0aE5hbWUgPSBzaVBhdGhOYW1lO1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZSB8fCB0eXBlTmFtZTtcbiAgICB0aGlzLnJvb3RQcm9wID0gbmV3IFByb3BPYmplY3Qoe1xuICAgICAgbmFtZTogdHlwZU5hbWUsXG4gICAgICBsYWJlbDogZGlzcGxheVR5cGVOYW1lLFxuICAgICAgY29tcG9uZW50VHlwZU5hbWU6IHR5cGVOYW1lLFxuICAgICAgcGFyZW50TmFtZTogXCJcIixcbiAgICB9KTtcbiAgICB0aGlzLm1ldGhvZHNQcm9wID0gbmV3IFByb3BPYmplY3Qoe1xuICAgICAgbmFtZTogYCR7dHlwZU5hbWV9YCxcbiAgICAgIGxhYmVsOiBgJHtkaXNwbGF5VHlwZU5hbWV9IE1ldGhvZHNgLFxuICAgICAgY29tcG9uZW50VHlwZU5hbWU6IHR5cGVOYW1lLFxuICAgICAgcGFyZW50TmFtZTogXCJcIixcbiAgICB9KTtcbiAgICB0aGlzLmFzc29jaWF0aW9ucyA9IG5ldyBBc3NvY2lhdGlvbkxpc3QoKTtcbiAgICB0aGlzLmludGVybmFsR3JhcGhxbCA9IHVuZGVmaW5lZDtcbiAgICB0aGlzLm12Y2MgPSBmYWxzZTtcbiAgfVxuXG4gIGdldCBmaWVsZHMoKTogQmFzZU9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgcmV0dXJuIHRoaXMucm9vdFByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGdldCBtZXRob2RzKCk6IEJhc2VPYmplY3RbXCJtZXRob2RzUHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIHJldHVybiB0aGlzLm1ldGhvZHNQcm9wLnByb3BlcnRpZXM7XG4gIH1cblxuICBnZXQgZ3JhcGhxbCgpOiBTaUdyYXBocWwge1xuICAgIGlmICh0aGlzLmludGVybmFsR3JhcGhxbCA9PSB1bmRlZmluZWQpIHtcbiAgICAgIHRoaXMuaW50ZXJuYWxHcmFwaHFsID0gbmV3IFNpR3JhcGhxbCh0aGlzKTtcbiAgICB9XG4gICAgcmV0dXJuIHRoaXMuaW50ZXJuYWxHcmFwaHFsO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImJhc2VPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgU3lzdGVtT2JqZWN0IGV4dGVuZHMgQmFzZU9iamVjdCB7XG4gIG5hdHVyYWxLZXkgPSBcIm5hbWVcIjtcbiAgbWlncmF0ZWFibGUgPSBmYWxzZTtcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICBzdXBlcihhcmdzKTtcbiAgICB0aGlzLnNldFN5c3RlbU9iamVjdERlZmF1bHRzKCk7XG4gIH1cblxuICBzZXRTeXN0ZW1PYmplY3REZWZhdWx0cygpOiB2b2lkIHtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiaWRcIixcbiAgICAgIGxhYmVsOiBgJHt0aGlzLmRpc3BsYXlUeXBlTmFtZX0gSURgLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwibmFtZVwiLFxuICAgICAgbGFiZWw6IGAke3RoaXMuZGlzcGxheVR5cGVOYW1lfSBOYW1lYCxcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImRpc3BsYXlOYW1lXCIsXG4gICAgICBsYWJlbDogYCR7dGhpcy5kaXNwbGF5VHlwZU5hbWV9IERpc3BsYXkgTmFtZWAsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJzaVN0b3JhYmxlXCIsXG4gICAgICBsYWJlbDogXCJTSSBTdG9yYWJsZVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IFwiZGF0YVN0b3JhYmxlXCIsXG4gICAgICAgIH07XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJzeXN0ZW1PYmplY3RcIjtcbiAgfVxuXG4gIGFkZEdldE1ldGhvZChhcmdzOiBBZGRNZXRob2RDb25zdHJ1Y3RvciA9IHt9KTogdm9pZCB7XG4gICAgLy8gZXNsaW50LWRpc2FibGUtbmV4dC1saW5lXG4gICAgY29uc3Qgc3lzdGVtT2JqZWN0ID0gdGhpcztcblxuICAgIHN5c3RlbU9iamVjdC5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImdldFwiLFxuICAgICAgbGFiZWw6IGBHZXQgYSAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9YCxcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLmlzUHJpdmF0ZSA9IGFyZ3MuaXNQcml2YXRlIHx8IGZhbHNlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImlkXCIsXG4gICAgICAgICAgbGFiZWw6IGAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9IElEYCxcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpdGVtXCIsXG4gICAgICAgICAgbGFiZWw6IGAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9IEl0ZW1gLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogc3lzdGVtT2JqZWN0LnR5cGVOYW1lLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cblxuICBhZGRMaXN0TWV0aG9kKGFyZ3M6IEFkZE1ldGhvZENvbnN0cnVjdG9yID0ge30pOiB2b2lkIHtcbiAgICAvLyBlc2xpbnQtZGlzYWJsZS1uZXh0LWxpbmVcbiAgICBjb25zdCBzeXN0ZW1PYmplY3QgPSB0aGlzO1xuICAgIHN5c3RlbU9iamVjdC5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImxpc3RcIixcbiAgICAgIGxhYmVsOiBgTGlzdCAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9YCxcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaXNQcml2YXRlID0gYXJncy5pc1ByaXZhdGUgfHwgZmFsc2U7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwicXVlcnlcIixcbiAgICAgICAgICBsYWJlbDogXCJRdWVyeVwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogXCJkYXRhUXVlcnlcIixcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZE51bWJlcih7XG4gICAgICAgICAgbmFtZTogXCJwYWdlU2l6ZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIlBhZ2UgU2l6ZVwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcE51bWJlcikge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5udW1iZXJLaW5kID0gXCJ1aW50MzJcIjtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJvcmRlckJ5XCIsXG4gICAgICAgICAgbGFiZWw6IFwiT3JkZXIgQnlcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJvcmRlckJ5RGlyZWN0aW9uXCIsXG4gICAgICAgICAgbGFiZWw6IFwiT3JkZXIgQnkgRGlyZWN0aW9uXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBcImRhdGFQYWdlVG9rZW5cIixcbiAgICAgICAgICAgICAgbmFtZXM6IFtcIm9yZGVyQnlEaXJlY3Rpb25cIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcInBhZ2VUb2tlblwiLFxuICAgICAgICAgIGxhYmVsOiBcIlBhZ2UgVG9rZW5cIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJzY29wZUJ5VGVuYW50SWRcIixcbiAgICAgICAgICBsYWJlbDogXCJTY29wZSBCeSBUZW5hbnQgSURcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiaXRlbXNcIixcbiAgICAgICAgICBsYWJlbDogXCJJdGVtc1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXBlYXRlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IHN5c3RlbU9iamVjdC50eXBlTmFtZSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGROdW1iZXIoe1xuICAgICAgICAgIG5hbWU6IFwidG90YWxDb3VudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIlRvdGFsIENvdW50XCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTnVtYmVyKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLm51bWJlcktpbmQgPSBcInVpbnQzMlwiO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJuZXh0UGFnZVRva2VuXCIsXG4gICAgICAgICAgbGFiZWw6IFwiTmV4dCBQYWdlIFRva2VuXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBDb21wb25lbnRPYmplY3QgZXh0ZW5kcyBTeXN0ZW1PYmplY3Qge1xuICBiYXNlVHlwZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICBjb25zdCB0eXBlTmFtZSA9IGAke2FyZ3MudHlwZU5hbWV9Q29tcG9uZW50YDtcbiAgICBjb25zdCBkaXNwbGF5VHlwZU5hbWUgPSBgJHthcmdzLmRpc3BsYXlUeXBlTmFtZX0gQ29tcG9uZW50YDtcbiAgICBzdXBlcih7XG4gICAgICB0eXBlTmFtZSxcbiAgICAgIGRpc3BsYXlUeXBlTmFtZSxcbiAgICAgIHNlcnZpY2VOYW1lOiBhcmdzLnNlcnZpY2VOYW1lLFxuICAgIH0pO1xuICAgIHRoaXMuYmFzZVR5cGVOYW1lID0gYXJncy50eXBlTmFtZTtcbiAgICB0aGlzLnNldENvbXBvbmVudERlZmF1bHRzKCk7XG4gIH1cblxuICBzZXRDb21wb25lbnREZWZhdWx0cygpOiB2b2lkIHtcbiAgICBjb25zdCBiYXNlVHlwZU5hbWUgPSB0aGlzLmJhc2VUeXBlTmFtZTtcblxuICAgIHRoaXMubWlncmF0ZWFibGUgPSB0cnVlO1xuXG4gICAgdGhpcy5hZGRHZXRNZXRob2QoKTtcbiAgICB0aGlzLmFkZExpc3RNZXRob2QoKTtcblxuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJkZXNjcmlwdGlvblwiLFxuICAgICAgbGFiZWw6IFwiQ29tcG9uZW50IERlc2NyaXB0aW9uXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkT2JqZWN0KHtcbiAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgIGxhYmVsOiBcIkNvbXBvbmVudCBDb25zdHJhaW50c1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wT2JqZWN0KSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIHAucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImNvbXBvbmVudE5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJDb21wb25lbnQgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJjb21wb25lbnREaXNwbGF5TmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkNvbXBvbmVudCBEaXNwbGF5IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBcImNvbXBvbmVudFNpUHJvcGVydGllc1wiLFxuICAgICAgICB9O1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkTWV0aG9kKHtcbiAgICAgIG5hbWU6IFwiY3JlYXRlXCIsXG4gICAgICBsYWJlbDogXCJDcmVhdGUgYSBDb21wb25lbnRcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLm11dGF0aW9uID0gdHJ1ZTtcbiAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICBwLmlzUHJpdmF0ZSA9IHRydWU7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwibmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkludGVncmF0aW9uIE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImRpc3BsYXlOYW1lXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSW50ZWdyYXRpb24gRGlzcGxheSBOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJkZXNjcmlwdGlvblwiLFxuICAgICAgICAgIGxhYmVsOiBcIkludGVncmF0aW9uIERpc3BsYXkgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgICAgICBsYWJlbDogXCJDb25zdHJhaW50c1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgICAgbmFtZXM6IFtcImNvbnN0cmFpbnRzXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJzaVByb3BlcnRpZXNcIixcbiAgICAgICAgICBsYWJlbDogXCJTaSBQcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IFwiY29tcG9uZW50U2lQcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpdGVtXCIsXG4gICAgICAgICAgbGFiZWw6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnQgSXRlbWAsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcInBpY2tcIixcbiAgICAgIGxhYmVsOiBcIlBpY2sgQ29tcG9uZW50XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJjb25zdHJhaW50c1wiLFxuICAgICAgICAgIGxhYmVsOiBcIkNvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpbXBsaWNpdENvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSW1wbGljaXQgQ29uc3RyYWludHNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImNvbXBvbmVudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIkNob3NlbiBDb21wb25lbnRcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cblxuICBnZXQgY29uc3RyYWludHMoKTogQ29tcG9uZW50T2JqZWN0W1wicm9vdFByb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICBjb25zdCBjb25zdHJhaW50UHJvcCA9IHRoaXMuZmllbGRzLmdldEVudHJ5KFwiY29uc3RyYWludHNcIikgYXMgUHJvcE9iamVjdDtcbiAgICByZXR1cm4gY29uc3RyYWludFByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJjb21wb25lbnRPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgRW50aXR5T2JqZWN0IGV4dGVuZHMgU3lzdGVtT2JqZWN0IHtcbiAgYmFzZVR5cGVOYW1lOiBzdHJpbmc7XG4gIGludGVncmF0aW9uU2VydmljZXM6IEludGVncmF0aW9uU2VydmljZVtdO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIGNvbnN0IHR5cGVOYW1lID0gYCR7YXJncy50eXBlTmFtZX1FbnRpdHlgO1xuICAgIGNvbnN0IGRpc3BsYXlUeXBlTmFtZSA9IGAke2FyZ3MuZGlzcGxheVR5cGVOYW1lfSBFbnRpdHlgO1xuICAgIHN1cGVyKHtcbiAgICAgIHR5cGVOYW1lLFxuICAgICAgZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5iYXNlVHlwZU5hbWUgPSBhcmdzLnR5cGVOYW1lO1xuICAgIHRoaXMuaW50ZWdyYXRpb25TZXJ2aWNlcyA9IFtdO1xuICAgIHRoaXMuc2V0RW50aXR5RGVmYXVsdHMoKTtcbiAgfVxuXG4gIHNldEVudGl0eURlZmF1bHRzKCk6IHZvaWQge1xuICAgIGNvbnN0IGJhc2VUeXBlTmFtZSA9IHRoaXMuYmFzZVR5cGVOYW1lO1xuXG4gICAgdGhpcy5tdmNjID0gdHJ1ZTtcblxuICAgIHRoaXMuYWRkR2V0TWV0aG9kKCk7XG4gICAgdGhpcy5hZGRMaXN0TWV0aG9kKCk7XG5cbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgIGxhYmVsOiBcIkVudGl0eSBEZXNjcmlwdGlvblwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJzaVByb3BlcnRpZXNcIixcbiAgICAgIGxhYmVsOiBcIlNJIFByb3BlcnRpZXNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogXCJlbnRpdHlTaVByb3BlcnRpZXNcIixcbiAgICAgICAgfTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZE9iamVjdCh7XG4gICAgICBuYW1lOiBcInByb3BlcnRpZXNcIixcbiAgICAgIGxhYmVsOiBcIlByb3BlcnRpZXNcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgIGxhYmVsOiBcIkNvbnN0cmFpbnRzXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJpbXBsaWNpdENvbnN0cmFpbnRzXCIsXG4gICAgICBsYWJlbDogXCJJbXBsaWNpdCBDb25zdHJhaW50c1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgbmFtZXM6IFtcImNvbnN0cmFpbnRzXCJdLFxuICAgICAgICB9O1xuICAgICAgfSxcbiAgICB9KTtcblxuICAgIHRoaXMubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJjcmVhdGVcIixcbiAgICAgIGxhYmVsOiBcIkNyZWF0ZSBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLm11dGF0aW9uID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJuYW1lXCIsXG4gICAgICAgICAgbGFiZWw6IFwiTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiZGlzcGxheU5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJEaXNwbGF5IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImRlc2NyaXB0aW9uXCIsXG4gICAgICAgICAgbGFiZWw6IFwiRGVzY3JpcHRpb25cIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcIndvcmtzcGFjZV9pZFwiLFxuICAgICAgICAgIGxhYmVsOiBgV29ya3NwYWNlIElEYCxcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInByb3BlcnRpZXNcIixcbiAgICAgICAgICBsYWJlbDogXCJQcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJwcm9wZXJ0aWVzXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJjb25zdHJhaW50c1wiLFxuICAgICAgICAgIGxhYmVsOiBcIkNvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpdGVtXCIsXG4gICAgICAgICAgbGFiZWw6IFwiJHtiYXNlVHlwZU5hbWV9RW50aXR5IEl0ZW1cIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJlbnRpdHlFdmVudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIkVudGl0eSBFdmVudFwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5RXZlbnRgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkQWN0aW9uKHtcbiAgICAgIG5hbWU6IFwic3luY1wiLFxuICAgICAgbGFiZWw6IFwiU3luYyBTdGF0ZVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wQWN0aW9uKSB7XG4gICAgICAgIHAubXV0YXRpb24gPSB0cnVlO1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAgZ2V0IHByb3BlcnRpZXMoKTogRW50aXR5T2JqZWN0W1wicm9vdFByb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICBjb25zdCBwcm9wID0gdGhpcy5maWVsZHMuZ2V0RW50cnkoXCJwcm9wZXJ0aWVzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJlbnRpdHlPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgRW50aXR5RXZlbnRPYmplY3QgZXh0ZW5kcyBTeXN0ZW1PYmplY3Qge1xuICBiYXNlVHlwZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICBjb25zdCB0eXBlTmFtZSA9IGAke2FyZ3MudHlwZU5hbWV9RW50aXR5RXZlbnRgO1xuICAgIGNvbnN0IGRpc3BsYXlUeXBlTmFtZSA9IGAke2FyZ3MuZGlzcGxheVR5cGVOYW1lfSBFbnRpdHlFdmVudGA7XG4gICAgc3VwZXIoe1xuICAgICAgdHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmJhc2VUeXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5zZXRFbnRpdHlFdmVudERlZmF1bHRzKCk7XG4gIH1cblxuICBzZXRFbnRpdHlFdmVudERlZmF1bHRzKCk6IHZvaWQge1xuICAgIGNvbnN0IGJhc2VUeXBlTmFtZSA9IHRoaXMuYmFzZVR5cGVOYW1lO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJhY3Rpb25OYW1lXCIsXG4gICAgICBsYWJlbDogXCJBY3Rpb24gTmFtZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiY3JlYXRlVGltZVwiLFxuICAgICAgbGFiZWw6IFwiQ3JlYXRpb24gVGltZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJ1cGRhdGVkVGltZVwiLFxuICAgICAgbGFiZWw6IFwiVXBkYXRlZCBUaW1lXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImZpbmFsVGltZVwiLFxuICAgICAgbGFiZWw6IFwiRmluYWwgVGltZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZEJvb2woe1xuICAgICAgbmFtZTogXCJzdWNjZXNzXCIsXG4gICAgICBsYWJlbDogXCJzdWNjZXNzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkQm9vbCh7XG4gICAgICBuYW1lOiBcImZpbmFsaXplZFwiLFxuICAgICAgbGFiZWw6IFwiRmluYWxpemVkXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcInVzZXJJZFwiLFxuICAgICAgbGFiZWw6IFwiVXNlciBJRFwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJvdXRwdXRMaW5lc1wiLFxuICAgICAgbGFiZWw6IFwiT3V0cHV0IExpbmVzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC5yZXBlYXRlZCA9IHRydWU7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImVycm9yTGluZXNcIixcbiAgICAgIGxhYmVsOiBcIkVycm9yIExpbmVzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC5yZXBlYXRlZCA9IHRydWU7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImVycm9yTWVzc2FnZVwiLFxuICAgICAgbGFiZWw6IFwiRXJyb3IgTWVzc2FnZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInByZXZpb3VzRW50aXR5XCIsXG4gICAgICBsYWJlbDogXCJQcmV2aW91cyBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcImlucHV0RW50aXR5XCIsXG4gICAgICBsYWJlbDogXCJJbnB1dCBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJvdXRwdXRFbnRpdHlcIixcbiAgICAgIGxhYmVsOiBcIk91dHB1dCBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IFwiZW50aXR5RXZlbnRTaVByb3BlcnRpZXNcIixcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLmFkZExpc3RNZXRob2QoKTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJlbnRpdHlFdmVudE9iamVjdFwiO1xuICB9XG59XG5cbmV4cG9ydCBpbnRlcmZhY2UgQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0Q29uc3RydWN0b3Ige1xuICB0eXBlTmFtZTogQmFzZU9iamVjdFtcInR5cGVOYW1lXCJdO1xuICBkaXNwbGF5VHlwZU5hbWU6IEJhc2VPYmplY3RbXCJkaXNwbGF5VHlwZU5hbWVcIl07XG4gIHNpUGF0aE5hbWU/OiBzdHJpbmc7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIG9wdGlvbnM/KGM6IENvbXBvbmVudEFuZEVudGl0eU9iamVjdCk6IHZvaWQ7XG59XG5cbmV4cG9ydCBjbGFzcyBDb21wb25lbnRBbmRFbnRpdHlPYmplY3Qge1xuICBjb21wb25lbnQ6IENvbXBvbmVudE9iamVjdDtcbiAgZW50aXR5OiBFbnRpdHlPYmplY3Q7XG4gIGVudGl0eUV2ZW50OiBFbnRpdHlFdmVudE9iamVjdDtcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBDb21wb25lbnRBbmRFbnRpdHlPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIHRoaXMuY29tcG9uZW50ID0gbmV3IENvbXBvbmVudE9iamVjdCh7XG4gICAgICB0eXBlTmFtZTogYXJncy50eXBlTmFtZSxcbiAgICAgIGRpc3BsYXlUeXBlTmFtZTogYXJncy5kaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzaVBhdGhOYW1lOiBhcmdzLnNpUGF0aE5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmVudGl0eSA9IG5ldyBFbnRpdHlPYmplY3Qoe1xuICAgICAgdHlwZU5hbWU6IGFyZ3MudHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWU6IGFyZ3MuZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2lQYXRoTmFtZTogYXJncy5zaVBhdGhOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5lbnRpdHlFdmVudCA9IG5ldyBFbnRpdHlFdmVudE9iamVjdCh7XG4gICAgICB0eXBlTmFtZTogYXJncy50eXBlTmFtZSxcbiAgICAgIGRpc3BsYXlUeXBlTmFtZTogYXJncy5kaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzaVBhdGhOYW1lOiBhcmdzLnNpUGF0aE5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgfVxuXG4gIGdldCBwcm9wZXJ0aWVzKCk6IEVudGl0eU9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgY29uc3QgcHJvcCA9IHRoaXMuZW50aXR5LmZpZWxkcy5nZXRFbnRyeShcInByb3BlcnRpZXNcIikgYXMgUHJvcE9iamVjdDtcbiAgICBwcm9wLnByb3BlcnRpZXMuYXV0b0NyZWF0ZUVkaXRzID0gdHJ1ZTtcbiAgICByZXR1cm4gcHJvcC5wcm9wZXJ0aWVzO1xuICB9XG5cbiAgZ2V0IGNvbnN0cmFpbnRzKCk6IENvbXBvbmVudE9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgY29uc3QgcHJvcCA9IHRoaXMuY29tcG9uZW50LmZpZWxkcy5nZXRFbnRyeShcImNvbnN0cmFpbnRzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxufVxuIl19