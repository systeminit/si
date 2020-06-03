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

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

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

      if (!this.typeName.endsWith("EntityEvent")) {
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
      }

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
            name: "workspaceId",
            label: "Workspace ID",
            options: function options(p) {
              p.required = true;
              p.hidden = true;
            }
          });
          p.request.properties.addText({
            name: "changeSetId",
            label: "Change Set ID",
            options: function options(p) {
              p.hidden = true;
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9zeXN0ZW1Db21wb25lbnQudHMiXSwibmFtZXMiOlsiQmFzZU9iamVjdCIsInR5cGVOYW1lIiwiZGlzcGxheVR5cGVOYW1lIiwic2VydmljZU5hbWUiLCJzaVBhdGhOYW1lIiwicm9vdFByb3AiLCJQcm9wT2JqZWN0IiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJwYXJlbnROYW1lIiwibWV0aG9kc1Byb3AiLCJhc3NvY2lhdGlvbnMiLCJBc3NvY2lhdGlvbkxpc3QiLCJpbnRlcm5hbEdyYXBocWwiLCJ1bmRlZmluZWQiLCJtdmNjIiwicHJvcGVydGllcyIsIlNpR3JhcGhxbCIsIlN5c3RlbU9iamVjdCIsImFyZ3MiLCJzZXRTeXN0ZW1PYmplY3REZWZhdWx0cyIsImZpZWxkcyIsImFkZFRleHQiLCJvcHRpb25zIiwicCIsInVuaXZlcnNhbCIsInJlYWRPbmx5IiwicmVxdWlyZWQiLCJlbmRzV2l0aCIsImFkZExpbmsiLCJoaWRkZW4iLCJsb29rdXAiLCJzeXN0ZW1PYmplY3QiLCJtZXRob2RzIiwiYWRkTWV0aG9kIiwiaXNQcml2YXRlIiwicmVxdWVzdCIsInJlcGx5IiwiYWRkTnVtYmVyIiwibnVtYmVyS2luZCIsIm5hbWVzIiwicmVwZWF0ZWQiLCJDb21wb25lbnRPYmplY3QiLCJiYXNlVHlwZU5hbWUiLCJzZXRDb21wb25lbnREZWZhdWx0cyIsIm1pZ3JhdGVhYmxlIiwiYWRkR2V0TWV0aG9kIiwiYWRkTGlzdE1ldGhvZCIsImFkZE9iamVjdCIsIm11dGF0aW9uIiwiY29uc3RyYWludFByb3AiLCJnZXRFbnRyeSIsIkVudGl0eU9iamVjdCIsImludGVncmF0aW9uU2VydmljZXMiLCJzZXRFbnRpdHlEZWZhdWx0cyIsImFkZEFjdGlvbiIsInByb3AiLCJFbnRpdHlFdmVudE9iamVjdCIsInNldEVudGl0eUV2ZW50RGVmYXVsdHMiLCJhZGRCb29sIiwiQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IiwiY29tcG9uZW50IiwiZW50aXR5IiwiZW50aXR5RXZlbnQiLCJhdXRvQ3JlYXRlRWRpdHMiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBRUE7O0FBTUE7O0FBQ0E7O0FBQ0E7Ozs7OztJQXFCYUEsVTtBQWFYLDRCQUswQjtBQUFBLFFBSnhCQyxRQUl3QixRQUp4QkEsUUFJd0I7QUFBQSxRQUh4QkMsZUFHd0IsUUFIeEJBLGVBR3dCO0FBQUEsUUFGeEJDLFdBRXdCLFFBRnhCQSxXQUV3QjtBQUFBLCtCQUR4QkMsVUFDd0I7QUFBQSxRQUR4QkEsVUFDd0IsZ0NBRFgsRUFDVztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQ3hCLFNBQUtILFFBQUwsR0FBZ0IsMkJBQVVBLFFBQVYsQ0FBaEI7QUFDQSxTQUFLQyxlQUFMLEdBQXVCQSxlQUF2QjtBQUNBLFNBQUtFLFVBQUwsR0FBa0JBLFVBQWxCO0FBQ0EsU0FBS0QsV0FBTCxHQUFtQkEsV0FBVyxJQUFJRixRQUFsQztBQUNBLFNBQUtJLFFBQUwsR0FBZ0IsSUFBSUMsb0JBQUosQ0FBZTtBQUM3QkMsTUFBQUEsSUFBSSxFQUFFTixRQUR1QjtBQUU3Qk8sTUFBQUEsS0FBSyxFQUFFTixlQUZzQjtBQUc3Qk8sTUFBQUEsaUJBQWlCLEVBQUVSLFFBSFU7QUFJN0JTLE1BQUFBLFVBQVUsRUFBRTtBQUppQixLQUFmLENBQWhCO0FBTUEsU0FBS0MsV0FBTCxHQUFtQixJQUFJTCxvQkFBSixDQUFlO0FBQ2hDQyxNQUFBQSxJQUFJLFlBQUtOLFFBQUwsQ0FENEI7QUFFaENPLE1BQUFBLEtBQUssWUFBS04sZUFBTCxhQUYyQjtBQUdoQ08sTUFBQUEsaUJBQWlCLEVBQUVSLFFBSGE7QUFJaENTLE1BQUFBLFVBQVUsRUFBRTtBQUpvQixLQUFmLENBQW5CO0FBTUEsU0FBS0UsWUFBTCxHQUFvQixJQUFJQyw2QkFBSixFQUFwQjtBQUNBLFNBQUtDLGVBQUwsR0FBdUJDLFNBQXZCO0FBQ0EsU0FBS0MsSUFBTCxHQUFZLEtBQVo7QUFDRDs7OzsyQkFpQmM7QUFDYixhQUFPLFlBQVA7QUFDRDs7O3dCQWpCa0Q7QUFDakQsYUFBTyxLQUFLWCxRQUFMLENBQWNZLFVBQXJCO0FBQ0Q7Ozt3QkFFc0Q7QUFDckQsYUFBTyxLQUFLTixXQUFMLENBQWlCTSxVQUF4QjtBQUNEOzs7d0JBRXdCO0FBQ3ZCLFVBQUksS0FBS0gsZUFBTCxJQUF3QkMsU0FBNUIsRUFBdUM7QUFDckMsYUFBS0QsZUFBTCxHQUF1QixJQUFJSSxrQkFBSixDQUFjLElBQWQsQ0FBdkI7QUFDRDs7QUFDRCxhQUFPLEtBQUtKLGVBQVo7QUFDRDs7Ozs7OztJQU9VSyxZOzs7OztBQUlYLHdCQUFZQyxJQUFaLEVBQXlDO0FBQUE7O0FBQUE7QUFDdkMsOEJBQU1BLElBQU47QUFEdUMsbUdBSDVCLE1BRzRCO0FBQUEsb0dBRjNCLEtBRTJCOztBQUV2QyxVQUFLQyx1QkFBTDs7QUFGdUM7QUFHeEM7Ozs7OENBRStCO0FBQzlCLFdBQUtDLE1BQUwsQ0FBWUMsT0FBWixDQUFvQjtBQUNsQmhCLFFBQUFBLElBQUksRUFBRSxJQURZO0FBRWxCQyxRQUFBQSxLQUFLLFlBQUssS0FBS04sZUFBVixRQUZhO0FBR2xCc0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHUDtBQUNUQSxVQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELFVBQUFBLENBQUMsQ0FBQ0UsUUFBRixHQUFhLElBQWI7QUFDQUYsVUFBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBUGlCLE9BQXBCOztBQVNBLFVBQUksQ0FBQyxLQUFLM0IsUUFBTCxDQUFjNEIsUUFBZCxDQUF1QixhQUF2QixDQUFMLEVBQTRDO0FBQzFDLGFBQUtQLE1BQUwsQ0FBWUMsT0FBWixDQUFvQjtBQUNsQmhCLFVBQUFBLElBQUksRUFBRSxNQURZO0FBRWxCQyxVQUFBQSxLQUFLLFlBQUssS0FBS04sZUFBVixVQUZhO0FBR2xCc0IsVUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHUDtBQUNUQSxZQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELFlBQUFBLENBQUMsQ0FBQ0UsUUFBRixHQUFhLElBQWI7QUFDQUYsWUFBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBUGlCLFNBQXBCO0FBU0EsYUFBS04sTUFBTCxDQUFZQyxPQUFaLENBQW9CO0FBQ2xCaEIsVUFBQUEsSUFBSSxFQUFFLGFBRFk7QUFFbEJDLFVBQUFBLEtBQUssWUFBSyxLQUFLTixlQUFWLGtCQUZhO0FBR2xCc0IsVUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHUDtBQUNUQSxZQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELFlBQUFBLENBQUMsQ0FBQ0UsUUFBRixHQUFhLElBQWI7QUFDQUYsWUFBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBUGlCLFNBQXBCO0FBU0Q7O0FBQ0QsV0FBS04sTUFBTCxDQUFZUSxPQUFaLENBQW9CO0FBQ2xCdkIsUUFBQUEsSUFBSSxFQUFFLFlBRFk7QUFFbEJDLFFBQUFBLEtBQUssRUFBRSxhQUZXO0FBR2xCZ0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHRztBQUNuQkEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNNLE1BQUYsR0FBVyxJQUFYO0FBQ0FOLFVBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixZQUFBQSxRQUFRLEVBQUU7QUFERCxXQUFYO0FBR0F3QixVQUFBQSxDQUFDLENBQUNHLFFBQUYsR0FBYSxJQUFiO0FBQ0Q7QUFWaUIsT0FBcEI7QUFZRDs7OzJCQUVjO0FBQ2IsYUFBTyxjQUFQO0FBQ0Q7OzttQ0FFbUQ7QUFBQSxVQUF2Q1IsSUFBdUMsdUVBQVYsRUFBVTtBQUNsRDtBQUNBLFVBQU1hLFlBQVksR0FBRyxJQUFyQjtBQUVBQSxNQUFBQSxZQUFZLENBQUNDLE9BQWIsQ0FBcUJDLFNBQXJCLENBQStCO0FBQzdCNUIsUUFBQUEsSUFBSSxFQUFFLEtBRHVCO0FBRTdCQyxRQUFBQSxLQUFLLGtCQUFXeUIsWUFBWSxDQUFDL0IsZUFBeEIsQ0FGd0I7QUFHN0JzQixRQUFBQSxPQUg2QixtQkFHckJDLENBSHFCLEVBR047QUFDckJBLFVBQUFBLENBQUMsQ0FBQ1csU0FBRixHQUFjaEIsSUFBSSxDQUFDZ0IsU0FBTCxJQUFrQixLQUFoQztBQUNBWCxVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJNLE9BQXJCLENBQTZCO0FBQzNCaEIsWUFBQUEsSUFBSSxFQUFFLElBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLFlBQUt5QixZQUFZLENBQUMvQixlQUFsQixRQUZzQjtBQUczQnNCLFlBQUFBLE9BSDJCLG1CQUduQkMsQ0FIbUIsRUFHaEI7QUFDVEEsY0FBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBTDBCLFdBQTdCO0FBT0FILFVBQUFBLENBQUMsQ0FBQ2EsS0FBRixDQUFRckIsVUFBUixDQUFtQmEsT0FBbkIsQ0FBMkI7QUFDekJ2QixZQUFBQSxJQUFJLEVBQUUsTUFEbUI7QUFFekJDLFlBQUFBLEtBQUssWUFBS3lCLFlBQVksQ0FBQy9CLGVBQWxCLFVBRm9CO0FBR3pCc0IsWUFBQUEsT0FIeUIsbUJBR2pCQyxDQUhpQixFQUdKO0FBQ25CQSxjQUFBQSxDQUFDLENBQUNPLE1BQUYsR0FBVztBQUNUL0IsZ0JBQUFBLFFBQVEsRUFBRWdDLFlBQVksQ0FBQ2hDO0FBRGQsZUFBWDtBQUdEO0FBUHdCLFdBQTNCO0FBU0Q7QUFyQjRCLE9BQS9CO0FBdUJEOzs7b0NBRW9EO0FBQUEsVUFBdkNtQixJQUF1Qyx1RUFBVixFQUFVO0FBQ25EO0FBQ0EsVUFBTWEsWUFBWSxHQUFHLElBQXJCO0FBQ0FBLE1BQUFBLFlBQVksQ0FBQ0MsT0FBYixDQUFxQkMsU0FBckIsQ0FBK0I7QUFDN0I1QixRQUFBQSxJQUFJLEVBQUUsTUFEdUI7QUFFN0JDLFFBQUFBLEtBQUssaUJBQVV5QixZQUFZLENBQUMvQixlQUF2QixDQUZ3QjtBQUc3QnNCLFFBQUFBLE9BSDZCLG1CQUdyQkMsQ0FIcUIsRUFHTjtBQUNyQkEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNXLFNBQUYsR0FBY2hCLElBQUksQ0FBQ2dCLFNBQUwsSUFBa0IsS0FBaEM7QUFDQVgsVUFBQUEsQ0FBQyxDQUFDWSxPQUFGLENBQVVwQixVQUFWLENBQXFCYSxPQUFyQixDQUE2QjtBQUMzQnZCLFlBQUFBLElBQUksRUFBRSxPQURxQjtBQUUzQkMsWUFBQUEsS0FBSyxFQUFFLE9BRm9CO0FBRzNCZ0IsWUFBQUEsT0FIMkIsbUJBR25CQyxDQUhtQixFQUdOO0FBQ25CQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELGNBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixnQkFBQUEsUUFBUSxFQUFFO0FBREQsZUFBWDtBQUdEO0FBUjBCLFdBQTdCO0FBVUF3QixVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJzQixTQUFyQixDQUErQjtBQUM3QmhDLFlBQUFBLElBQUksRUFBRSxVQUR1QjtBQUU3QkMsWUFBQUEsS0FBSyxFQUFFLFdBRnNCO0FBRzdCZ0IsWUFBQUEsT0FINkIsbUJBR3JCQyxDQUhxQixFQUdOO0FBQ3JCQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELGNBQUFBLENBQUMsQ0FBQ2UsVUFBRixHQUFlLFFBQWY7QUFDRDtBQU40QixXQUEvQjtBQVFBZixVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJNLE9BQXJCLENBQTZCO0FBQzNCaEIsWUFBQUEsSUFBSSxFQUFFLFNBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsVUFGb0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR2hCO0FBQ1RBLGNBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDRDtBQUwwQixXQUE3QjtBQU9BRCxVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJhLE9BQXJCLENBQTZCO0FBQzNCdkIsWUFBQUEsSUFBSSxFQUFFLGtCQURxQjtBQUUzQkMsWUFBQUEsS0FBSyxFQUFFLG9CQUZvQjtBQUczQmdCLFlBQUFBLE9BSDJCLG1CQUduQkMsQ0FIbUIsRUFHTjtBQUNuQkEsY0FBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxjQUFBQSxDQUFDLENBQUNPLE1BQUYsR0FBVztBQUNUL0IsZ0JBQUFBLFFBQVEsRUFBRSxlQUREO0FBRVR3QyxnQkFBQUEsS0FBSyxFQUFFLENBQUMsa0JBQUQ7QUFGRSxlQUFYO0FBSUQ7QUFUMEIsV0FBN0I7QUFXQWhCLFVBQUFBLENBQUMsQ0FBQ1ksT0FBRixDQUFVcEIsVUFBVixDQUFxQk0sT0FBckIsQ0FBNkI7QUFDM0JoQixZQUFBQSxJQUFJLEVBQUUsV0FEcUI7QUFFM0JDLFlBQUFBLEtBQUssRUFBRSxZQUZvQjtBQUczQmdCLFlBQUFBLE9BSDJCLG1CQUduQkMsQ0FIbUIsRUFHaEI7QUFDVEEsY0FBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNEO0FBTDBCLFdBQTdCO0FBT0FELFVBQUFBLENBQUMsQ0FBQ1ksT0FBRixDQUFVcEIsVUFBVixDQUFxQk0sT0FBckIsQ0FBNkI7QUFDM0JoQixZQUFBQSxJQUFJLEVBQUUsaUJBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsb0JBRm9CO0FBRzNCZ0IsWUFBQUEsT0FIMkIsbUJBR25CQyxDQUhtQixFQUdoQjtBQUNUQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0Q7QUFMMEIsV0FBN0I7QUFPQUQsVUFBQUEsQ0FBQyxDQUFDYSxLQUFGLENBQVFyQixVQUFSLENBQW1CYSxPQUFuQixDQUEyQjtBQUN6QnZCLFlBQUFBLElBQUksRUFBRSxPQURtQjtBQUV6QkMsWUFBQUEsS0FBSyxFQUFFLE9BRmtCO0FBR3pCZ0IsWUFBQUEsT0FIeUIsbUJBR2pCQyxDQUhpQixFQUdKO0FBQ25CQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELGNBQUFBLENBQUMsQ0FBQ0csUUFBRixHQUFhLElBQWI7QUFDQUgsY0FBQUEsQ0FBQyxDQUFDaUIsUUFBRixHQUFhLElBQWI7QUFDQWpCLGNBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixnQkFBQUEsUUFBUSxFQUFFZ0MsWUFBWSxDQUFDaEM7QUFEZCxlQUFYO0FBR0Q7QUFWd0IsV0FBM0I7QUFZQXdCLFVBQUFBLENBQUMsQ0FBQ2EsS0FBRixDQUFRckIsVUFBUixDQUFtQnNCLFNBQW5CLENBQTZCO0FBQzNCaEMsWUFBQUEsSUFBSSxFQUFFLFlBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsYUFGb0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR0o7QUFDckJBLGNBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsY0FBQUEsQ0FBQyxDQUFDZSxVQUFGLEdBQWUsUUFBZjtBQUNEO0FBTjBCLFdBQTdCO0FBUUFmLFVBQUFBLENBQUMsQ0FBQ2EsS0FBRixDQUFRckIsVUFBUixDQUFtQk0sT0FBbkIsQ0FBMkI7QUFDekJoQixZQUFBQSxJQUFJLEVBQUUsZUFEbUI7QUFFekJDLFlBQUFBLEtBQUssRUFBRSxpQkFGa0I7QUFHekJnQixZQUFBQSxPQUh5QixtQkFHakJDLENBSGlCLEVBR2Q7QUFDVEEsY0FBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNEO0FBTHdCLFdBQTNCO0FBT0Q7QUFuRjRCLE9BQS9CO0FBcUZEOzs7RUE5SytCMUIsVTs7OztJQWlMckIyQyxlOzs7OztBQUdYLDJCQUFZdkIsSUFBWixFQUF5QztBQUFBOztBQUFBO0FBQ3ZDLFFBQU1uQixRQUFRLGFBQU1tQixJQUFJLENBQUNuQixRQUFYLGNBQWQ7QUFDQSxRQUFNQyxlQUFlLGFBQU1rQixJQUFJLENBQUNsQixlQUFYLGVBQXJCO0FBQ0EsZ0NBQU07QUFDSkQsTUFBQUEsUUFBUSxFQUFSQSxRQURJO0FBRUpDLE1BQUFBLGVBQWUsRUFBZkEsZUFGSTtBQUdKQyxNQUFBQSxXQUFXLEVBQUVpQixJQUFJLENBQUNqQjtBQUhkLEtBQU47QUFIdUM7QUFRdkMsV0FBS3lDLFlBQUwsR0FBb0J4QixJQUFJLENBQUNuQixRQUF6Qjs7QUFDQSxXQUFLNEMsb0JBQUw7O0FBVHVDO0FBVXhDOzs7OzJDQUU0QjtBQUMzQixVQUFNRCxZQUFZLEdBQUcsS0FBS0EsWUFBMUI7QUFFQSxXQUFLRSxXQUFMLEdBQW1CLElBQW5CO0FBRUEsV0FBS0MsWUFBTDtBQUNBLFdBQUtDLGFBQUw7QUFFQSxXQUFLMUIsTUFBTCxDQUFZQyxPQUFaLENBQW9CO0FBQ2xCaEIsUUFBQUEsSUFBSSxFQUFFLGFBRFk7QUFFbEJDLFFBQUFBLEtBQUssRUFBRSx1QkFGVztBQUdsQmdCLFFBQUFBLE9BSGtCLG1CQUdWQyxDQUhVLEVBR1A7QUFDVEEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNHLFFBQUYsR0FBYSxJQUFiO0FBQ0Q7QUFOaUIsT0FBcEI7QUFRQSxXQUFLTixNQUFMLENBQVkyQixTQUFaLENBQXNCO0FBQ3BCMUMsUUFBQUEsSUFBSSxFQUFFLGFBRGM7QUFFcEJDLFFBQUFBLEtBQUssRUFBRSx1QkFGYTtBQUdwQmdCLFFBQUFBLE9BSG9CLG1CQUdaQyxDQUhZLEVBR0c7QUFDckJBLFVBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsVUFBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNBSCxVQUFBQSxDQUFDLENBQUNSLFVBQUYsQ0FBYU0sT0FBYixDQUFxQjtBQUNuQmhCLFlBQUFBLElBQUksRUFBRSxlQURhO0FBRW5CQyxZQUFBQSxLQUFLLEVBQUUsZ0JBRlk7QUFHbkJnQixZQUFBQSxPQUhtQixtQkFHWEMsQ0FIVyxFQUdSO0FBQ1RBLGNBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDRDtBQUxrQixXQUFyQjtBQU9BRCxVQUFBQSxDQUFDLENBQUNSLFVBQUYsQ0FBYU0sT0FBYixDQUFxQjtBQUNuQmhCLFlBQUFBLElBQUksRUFBRSxzQkFEYTtBQUVuQkMsWUFBQUEsS0FBSyxFQUFFLHdCQUZZO0FBR25CZ0IsWUFBQUEsT0FIbUIsbUJBR1hDLENBSFcsRUFHUjtBQUNUQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0Q7QUFMa0IsV0FBckI7QUFPRDtBQXBCbUIsT0FBdEI7QUFzQkEsV0FBS0osTUFBTCxDQUFZUSxPQUFaLENBQW9CO0FBQ2xCdkIsUUFBQUEsSUFBSSxFQUFFLGNBRFk7QUFFbEJDLFFBQUFBLEtBQUssRUFBRSxlQUZXO0FBR2xCZ0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHRztBQUNuQkEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNPLE1BQUYsR0FBVztBQUNUL0IsWUFBQUEsUUFBUSxFQUFFO0FBREQsV0FBWDtBQUdBd0IsVUFBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBVGlCLE9BQXBCO0FBWUEsV0FBS00sT0FBTCxDQUFhQyxTQUFiLENBQXVCO0FBQ3JCNUIsUUFBQUEsSUFBSSxFQUFFLFFBRGU7QUFFckJDLFFBQUFBLEtBQUssRUFBRSxvQkFGYztBQUdyQmdCLFFBQUFBLE9BSHFCLG1CQUdiQyxDQUhhLEVBR0U7QUFDckJBLFVBQUFBLENBQUMsQ0FBQ3lCLFFBQUYsR0FBYSxJQUFiO0FBQ0F6QixVQUFBQSxDQUFDLENBQUNNLE1BQUYsR0FBVyxJQUFYO0FBQ0FOLFVBQUFBLENBQUMsQ0FBQ1csU0FBRixHQUFjLElBQWQ7QUFDQVgsVUFBQUEsQ0FBQyxDQUFDWSxPQUFGLENBQVVwQixVQUFWLENBQXFCTSxPQUFyQixDQUE2QjtBQUMzQmhCLFlBQUFBLElBQUksRUFBRSxNQURxQjtBQUUzQkMsWUFBQUEsS0FBSyxFQUFFLGtCQUZvQjtBQUczQmdCLFlBQUFBLE9BSDJCLG1CQUduQkMsQ0FIbUIsRUFHaEI7QUFDVEEsY0FBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBTDBCLFdBQTdCO0FBT0FILFVBQUFBLENBQUMsQ0FBQ1ksT0FBRixDQUFVcEIsVUFBVixDQUFxQk0sT0FBckIsQ0FBNkI7QUFDM0JoQixZQUFBQSxJQUFJLEVBQUUsYUFEcUI7QUFFM0JDLFlBQUFBLEtBQUssRUFBRSwwQkFGb0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR2hCO0FBQ1RBLGNBQUFBLENBQUMsQ0FBQ0csUUFBRixHQUFhLElBQWI7QUFDRDtBQUwwQixXQUE3QjtBQU9BSCxVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJNLE9BQXJCLENBQTZCO0FBQzNCaEIsWUFBQUEsSUFBSSxFQUFFLGFBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsMEJBRm9CO0FBRzNCZ0IsWUFBQUEsT0FIMkIsbUJBR25CQyxDQUhtQixFQUdoQjtBQUNUQSxjQUFBQSxDQUFDLENBQUNHLFFBQUYsR0FBYSxJQUFiO0FBQ0Q7QUFMMEIsV0FBN0I7QUFPQUgsVUFBQUEsQ0FBQyxDQUFDWSxPQUFGLENBQVVwQixVQUFWLENBQXFCYSxPQUFyQixDQUE2QjtBQUMzQnZCLFlBQUFBLElBQUksRUFBRSxhQURxQjtBQUUzQkMsWUFBQUEsS0FBSyxFQUFFLGFBRm9CO0FBRzNCZ0IsWUFBQUEsT0FIMkIsbUJBR25CQyxDQUhtQixFQUdOO0FBQ25CQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELGNBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixnQkFBQUEsUUFBUSxZQUFLMkMsWUFBTCxjQURDO0FBRVRILGdCQUFBQSxLQUFLLEVBQUUsQ0FBQyxhQUFEO0FBRkUsZUFBWDtBQUlEO0FBVDBCLFdBQTdCO0FBV0FoQixVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJhLE9BQXJCLENBQTZCO0FBQzNCdkIsWUFBQUEsSUFBSSxFQUFFLGNBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsZUFGb0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR047QUFDbkJBLGNBQUFBLENBQUMsQ0FBQ0csUUFBRixHQUFhLElBQWI7QUFDQUgsY0FBQUEsQ0FBQyxDQUFDTyxNQUFGLEdBQVc7QUFDVC9CLGdCQUFBQSxRQUFRLEVBQUU7QUFERCxlQUFYO0FBR0Q7QUFSMEIsV0FBN0I7QUFVQXdCLFVBQUFBLENBQUMsQ0FBQ2EsS0FBRixDQUFRckIsVUFBUixDQUFtQmEsT0FBbkIsQ0FBMkI7QUFDekJ2QixZQUFBQSxJQUFJLEVBQUUsTUFEbUI7QUFFekJDLFlBQUFBLEtBQUssWUFBS29DLFlBQUwsbUJBRm9CO0FBR3pCcEIsWUFBQUEsT0FIeUIsbUJBR2pCQyxDQUhpQixFQUdKO0FBQ25CQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELGNBQUFBLENBQUMsQ0FBQ0UsUUFBRixHQUFhLElBQWI7QUFDQUYsY0FBQUEsQ0FBQyxDQUFDTyxNQUFGLEdBQVc7QUFDVC9CLGdCQUFBQSxRQUFRLFlBQUsyQyxZQUFMO0FBREMsZUFBWDtBQUdEO0FBVHdCLFdBQTNCO0FBV0Q7QUE1RG9CLE9BQXZCO0FBOERBLFdBQUtWLE9BQUwsQ0FBYUMsU0FBYixDQUF1QjtBQUNyQjVCLFFBQUFBLElBQUksRUFBRSxNQURlO0FBRXJCQyxRQUFBQSxLQUFLLEVBQUUsZ0JBRmM7QUFHckJnQixRQUFBQSxPQUhxQixtQkFHYkMsQ0FIYSxFQUdFO0FBQ3JCQSxVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJhLE9BQXJCLENBQTZCO0FBQzNCdkIsWUFBQUEsSUFBSSxFQUFFLGFBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsYUFGb0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR047QUFDbkJBLGNBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsY0FBQUEsQ0FBQyxDQUFDTyxNQUFGLEdBQVc7QUFDVC9CLGdCQUFBQSxRQUFRLFlBQUsyQyxZQUFMLGNBREM7QUFFVEgsZ0JBQUFBLEtBQUssRUFBRSxDQUFDLGFBQUQ7QUFGRSxlQUFYO0FBSUQ7QUFUMEIsV0FBN0I7QUFXQWhCLFVBQUFBLENBQUMsQ0FBQ2EsS0FBRixDQUFRckIsVUFBUixDQUFtQmEsT0FBbkIsQ0FBMkI7QUFDekJ2QixZQUFBQSxJQUFJLEVBQUUscUJBRG1CO0FBRXpCQyxZQUFBQSxLQUFLLEVBQUUsc0JBRmtCO0FBR3pCZ0IsWUFBQUEsT0FIeUIsbUJBR2pCQyxDQUhpQixFQUdKO0FBQ25CQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELGNBQUFBLENBQUMsQ0FBQ0csUUFBRixHQUFhLElBQWI7QUFDQUgsY0FBQUEsQ0FBQyxDQUFDTyxNQUFGLEdBQVc7QUFDVC9CLGdCQUFBQSxRQUFRLFlBQUsyQyxZQUFMLGNBREM7QUFFVEgsZ0JBQUFBLEtBQUssRUFBRSxDQUFDLGFBQUQ7QUFGRSxlQUFYO0FBSUQ7QUFWd0IsV0FBM0I7QUFZQWhCLFVBQUFBLENBQUMsQ0FBQ2EsS0FBRixDQUFRckIsVUFBUixDQUFtQmEsT0FBbkIsQ0FBMkI7QUFDekJ2QixZQUFBQSxJQUFJLEVBQUUsV0FEbUI7QUFFekJDLFlBQUFBLEtBQUssRUFBRSxrQkFGa0I7QUFHekJnQixZQUFBQSxPQUh5QixtQkFHakJDLENBSGlCLEVBR0o7QUFDbkJBLGNBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsY0FBQUEsQ0FBQyxDQUFDTyxNQUFGLEdBQVc7QUFDVC9CLGdCQUFBQSxRQUFRLFlBQUsyQyxZQUFMO0FBREMsZUFBWDtBQUdEO0FBUndCLFdBQTNCO0FBVUQ7QUFyQ29CLE9BQXZCO0FBdUNEOzs7MkJBT2M7QUFDYixhQUFPLGlCQUFQO0FBQ0Q7Ozt3QkFQNEQ7QUFDM0QsVUFBTU8sY0FBYyxHQUFHLEtBQUs3QixNQUFMLENBQVk4QixRQUFaLENBQXFCLGFBQXJCLENBQXZCO0FBQ0EsYUFBT0QsY0FBYyxDQUFDbEMsVUFBdEI7QUFDRDs7O0VBM0trQ0UsWTs7OztJQWtMeEJrQyxZOzs7OztBQUlYLHdCQUFZakMsSUFBWixFQUF5QztBQUFBOztBQUFBO0FBQ3ZDLFFBQU1uQixRQUFRLGFBQU1tQixJQUFJLENBQUNuQixRQUFYLFdBQWQ7QUFDQSxRQUFNQyxlQUFlLGFBQU1rQixJQUFJLENBQUNsQixlQUFYLFlBQXJCO0FBQ0EsZ0NBQU07QUFDSkQsTUFBQUEsUUFBUSxFQUFSQSxRQURJO0FBRUpDLE1BQUFBLGVBQWUsRUFBZkEsZUFGSTtBQUdKQyxNQUFBQSxXQUFXLEVBQUVpQixJQUFJLENBQUNqQjtBQUhkLEtBQU47QUFIdUM7QUFBQTtBQVF2QyxXQUFLeUMsWUFBTCxHQUFvQnhCLElBQUksQ0FBQ25CLFFBQXpCO0FBQ0EsV0FBS3FELG1CQUFMLEdBQTJCLEVBQTNCOztBQUNBLFdBQUtDLGlCQUFMOztBQVZ1QztBQVd4Qzs7Ozt3Q0FFeUI7QUFDeEIsVUFBTVgsWUFBWSxHQUFHLEtBQUtBLFlBQTFCO0FBRUEsV0FBSzVCLElBQUwsR0FBWSxJQUFaO0FBRUEsV0FBSytCLFlBQUw7QUFDQSxXQUFLQyxhQUFMO0FBRUEsV0FBSzFCLE1BQUwsQ0FBWUMsT0FBWixDQUFvQjtBQUNsQmhCLFFBQUFBLElBQUksRUFBRSxhQURZO0FBRWxCQyxRQUFBQSxLQUFLLEVBQUUsb0JBRlc7QUFHbEJnQixRQUFBQSxPQUhrQixtQkFHVkMsQ0FIVSxFQUdQO0FBQ1RBLFVBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsVUFBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBTmlCLE9BQXBCO0FBUUEsV0FBS04sTUFBTCxDQUFZUSxPQUFaLENBQW9CO0FBQ2xCdkIsUUFBQUEsSUFBSSxFQUFFLGNBRFk7QUFFbEJDLFFBQUFBLEtBQUssRUFBRSxlQUZXO0FBR2xCZ0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHRztBQUNuQkEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNPLE1BQUYsR0FBVztBQUNUL0IsWUFBQUEsUUFBUSxFQUFFO0FBREQsV0FBWDtBQUdBd0IsVUFBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBVGlCLE9BQXBCO0FBV0EsV0FBS04sTUFBTCxDQUFZMkIsU0FBWixDQUFzQjtBQUNwQjFDLFFBQUFBLElBQUksRUFBRSxZQURjO0FBRXBCQyxRQUFBQSxLQUFLLEVBQUUsWUFGYTtBQUdwQmdCLFFBQUFBLE9BSG9CLG1CQUdaQyxDQUhZLEVBR1Q7QUFDVEEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNHLFFBQUYsR0FBYSxJQUFiO0FBQ0Q7QUFObUIsT0FBdEI7QUFRQSxXQUFLTixNQUFMLENBQVlRLE9BQVosQ0FBb0I7QUFDbEJ2QixRQUFBQSxJQUFJLEVBQUUsYUFEWTtBQUVsQkMsUUFBQUEsS0FBSyxFQUFFLGFBRlc7QUFHbEJnQixRQUFBQSxPQUhrQixtQkFHVkMsQ0FIVSxFQUdHO0FBQ25CQSxVQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELFVBQUFBLENBQUMsQ0FBQ0UsUUFBRixHQUFhLElBQWI7QUFDQUYsVUFBQUEsQ0FBQyxDQUFDTyxNQUFGLEdBQVc7QUFDVC9CLFlBQUFBLFFBQVEsWUFBSzJDLFlBQUwsY0FEQztBQUVUSCxZQUFBQSxLQUFLLEVBQUUsQ0FBQyxhQUFEO0FBRkUsV0FBWDtBQUlEO0FBVmlCLE9BQXBCO0FBWUEsV0FBS25CLE1BQUwsQ0FBWVEsT0FBWixDQUFvQjtBQUNsQnZCLFFBQUFBLElBQUksRUFBRSxxQkFEWTtBQUVsQkMsUUFBQUEsS0FBSyxFQUFFLHNCQUZXO0FBR2xCZ0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHRztBQUNuQkEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNFLFFBQUYsR0FBYSxJQUFiO0FBQ0FGLFVBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixZQUFBQSxRQUFRLFlBQUsyQyxZQUFMLGNBREM7QUFFVEgsWUFBQUEsS0FBSyxFQUFFLENBQUMsYUFBRDtBQUZFLFdBQVg7QUFJRDtBQVZpQixPQUFwQjtBQWFBLFdBQUtQLE9BQUwsQ0FBYUMsU0FBYixDQUF1QjtBQUNyQjVCLFFBQUFBLElBQUksRUFBRSxRQURlO0FBRXJCQyxRQUFBQSxLQUFLLEVBQUUsZUFGYztBQUdyQmdCLFFBQUFBLE9BSHFCLG1CQUdiQyxDQUhhLEVBR0U7QUFDckJBLFVBQUFBLENBQUMsQ0FBQ3lCLFFBQUYsR0FBYSxJQUFiO0FBQ0F6QixVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJNLE9BQXJCLENBQTZCO0FBQzNCaEIsWUFBQUEsSUFBSSxFQUFFLE1BRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsTUFGb0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR2hCO0FBQ1RBLGNBQUFBLENBQUMsQ0FBQ0csUUFBRixHQUFhLElBQWI7QUFDQUgsY0FBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNEO0FBTjBCLFdBQTdCO0FBUUFELFVBQUFBLENBQUMsQ0FBQ1ksT0FBRixDQUFVcEIsVUFBVixDQUFxQk0sT0FBckIsQ0FBNkI7QUFDM0JoQixZQUFBQSxJQUFJLEVBQUUsYUFEcUI7QUFFM0JDLFlBQUFBLEtBQUssRUFBRSxjQUZvQjtBQUczQmdCLFlBQUFBLE9BSDJCLG1CQUduQkMsQ0FIbUIsRUFHaEI7QUFDVEEsY0FBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNBSCxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0Q7QUFOMEIsV0FBN0I7QUFRQUQsVUFBQUEsQ0FBQyxDQUFDWSxPQUFGLENBQVVwQixVQUFWLENBQXFCTSxPQUFyQixDQUE2QjtBQUMzQmhCLFlBQUFBLElBQUksRUFBRSxhQURxQjtBQUUzQkMsWUFBQUEsS0FBSyxFQUFFLGFBRm9CO0FBRzNCZ0IsWUFBQUEsT0FIMkIsbUJBR25CQyxDQUhtQixFQUdoQjtBQUNUQSxjQUFBQSxDQUFDLENBQUNHLFFBQUYsR0FBYSxJQUFiO0FBQ0FILGNBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDRDtBQU4wQixXQUE3QjtBQVFBRCxVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJNLE9BQXJCLENBQTZCO0FBQzNCaEIsWUFBQUEsSUFBSSxFQUFFLGFBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLGdCQUZzQjtBQUczQmdCLFlBQUFBLE9BSDJCLG1CQUduQkMsQ0FIbUIsRUFHaEI7QUFDVEEsY0FBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNBSCxjQUFBQSxDQUFDLENBQUNNLE1BQUYsR0FBVyxJQUFYO0FBQ0Q7QUFOMEIsV0FBN0I7QUFRQU4sVUFBQUEsQ0FBQyxDQUFDWSxPQUFGLENBQVVwQixVQUFWLENBQXFCTSxPQUFyQixDQUE2QjtBQUMzQmhCLFlBQUFBLElBQUksRUFBRSxhQURxQjtBQUUzQkMsWUFBQUEsS0FBSyxpQkFGc0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR2hCO0FBQ1RBLGNBQUFBLENBQUMsQ0FBQ00sTUFBRixHQUFXLElBQVg7QUFDRDtBQUwwQixXQUE3QjtBQU9BTixVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJhLE9BQXJCLENBQTZCO0FBQzNCdkIsWUFBQUEsSUFBSSxFQUFFLFlBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsWUFGb0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR047QUFDbkJBLGNBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsY0FBQUEsQ0FBQyxDQUFDRSxRQUFGLEdBQWEsSUFBYjtBQUNBRixjQUFBQSxDQUFDLENBQUNHLFFBQUYsR0FBYSxJQUFiO0FBQ0FILGNBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixnQkFBQUEsUUFBUSxZQUFLMkMsWUFBTCxXQURDO0FBRVRILGdCQUFBQSxLQUFLLEVBQUUsQ0FBQyxZQUFEO0FBRkUsZUFBWDtBQUlEO0FBWDBCLFdBQTdCO0FBYUFoQixVQUFBQSxDQUFDLENBQUNZLE9BQUYsQ0FBVXBCLFVBQVYsQ0FBcUJhLE9BQXJCLENBQTZCO0FBQzNCdkIsWUFBQUEsSUFBSSxFQUFFLGFBRHFCO0FBRTNCQyxZQUFBQSxLQUFLLEVBQUUsYUFGb0I7QUFHM0JnQixZQUFBQSxPQUgyQixtQkFHbkJDLENBSG1CLEVBR047QUFDbkJBLGNBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsY0FBQUEsQ0FBQyxDQUFDRSxRQUFGLEdBQWEsSUFBYjtBQUNBRixjQUFBQSxDQUFDLENBQUNPLE1BQUYsR0FBVztBQUNUL0IsZ0JBQUFBLFFBQVEsWUFBSzJDLFlBQUwsY0FEQztBQUVUSCxnQkFBQUEsS0FBSyxFQUFFLENBQUMsYUFBRDtBQUZFLGVBQVg7QUFJRDtBQVYwQixXQUE3QjtBQVlBaEIsVUFBQUEsQ0FBQyxDQUFDYSxLQUFGLENBQVFyQixVQUFSLENBQW1CYSxPQUFuQixDQUEyQjtBQUN6QnZCLFlBQUFBLElBQUksRUFBRSxNQURtQjtBQUV6QkMsWUFBQUEsS0FBSyxFQUFFLDRCQUZrQjtBQUd6QmdCLFlBQUFBLE9BSHlCLG1CQUdqQkMsQ0FIaUIsRUFHSjtBQUNuQkEsY0FBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxjQUFBQSxDQUFDLENBQUNFLFFBQUYsR0FBYSxJQUFiO0FBQ0FGLGNBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixnQkFBQUEsUUFBUSxZQUFLMkMsWUFBTDtBQURDLGVBQVg7QUFHRDtBQVR3QixXQUEzQjtBQVdBbkIsVUFBQUEsQ0FBQyxDQUFDYSxLQUFGLENBQVFyQixVQUFSLENBQW1CYSxPQUFuQixDQUEyQjtBQUN6QnZCLFlBQUFBLElBQUksRUFBRSxhQURtQjtBQUV6QkMsWUFBQUEsS0FBSyxFQUFFLGNBRmtCO0FBR3pCZ0IsWUFBQUEsT0FIeUIsbUJBR2pCQyxDQUhpQixFQUdKO0FBQ25CQSxjQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELGNBQUFBLENBQUMsQ0FBQ0UsUUFBRixHQUFhLElBQWI7QUFDQUYsY0FBQUEsQ0FBQyxDQUFDTyxNQUFGLEdBQVc7QUFDVC9CLGdCQUFBQSxRQUFRLFlBQUsyQyxZQUFMO0FBREMsZUFBWDtBQUdEO0FBVHdCLFdBQTNCO0FBV0Q7QUEzRm9CLE9BQXZCO0FBOEZBLFdBQUtWLE9BQUwsQ0FBYXNCLFNBQWIsQ0FBdUI7QUFDckJqRCxRQUFBQSxJQUFJLEVBQUUsTUFEZTtBQUVyQkMsUUFBQUEsS0FBSyxFQUFFLFlBRmM7QUFHckJnQixRQUFBQSxPQUhxQixtQkFHYkMsQ0FIYSxFQUdFO0FBQ3JCQSxVQUFBQSxDQUFDLENBQUN5QixRQUFGLEdBQWEsSUFBYjtBQUNBekIsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNEO0FBTm9CLE9BQXZCO0FBUUQ7OzsyQkFPYztBQUNiLGFBQU8sY0FBUDtBQUNEOzs7d0JBUHdEO0FBQ3ZELFVBQU0rQixJQUFJLEdBQUcsS0FBS25DLE1BQUwsQ0FBWThCLFFBQVosQ0FBcUIsWUFBckIsQ0FBYjtBQUNBLGFBQU9LLElBQUksQ0FBQ3hDLFVBQVo7QUFDRDs7O0VBeEwrQkUsWTs7OztJQStMckJ1QyxpQjs7Ozs7QUFHWCw2QkFBWXRDLElBQVosRUFBeUM7QUFBQTs7QUFBQTtBQUN2QyxRQUFNbkIsUUFBUSxhQUFNbUIsSUFBSSxDQUFDbkIsUUFBWCxnQkFBZDtBQUNBLFFBQU1DLGVBQWUsYUFBTWtCLElBQUksQ0FBQ2xCLGVBQVgsaUJBQXJCO0FBQ0EsZ0NBQU07QUFDSkQsTUFBQUEsUUFBUSxFQUFSQSxRQURJO0FBRUpDLE1BQUFBLGVBQWUsRUFBZkEsZUFGSTtBQUdKQyxNQUFBQSxXQUFXLEVBQUVpQixJQUFJLENBQUNqQjtBQUhkLEtBQU47QUFIdUM7QUFRdkMsV0FBS3lDLFlBQUwsR0FBb0J4QixJQUFJLENBQUNuQixRQUF6Qjs7QUFDQSxXQUFLMEQsc0JBQUw7O0FBVHVDO0FBVXhDOzs7OzZDQUU4QjtBQUM3QixVQUFNZixZQUFZLEdBQUcsS0FBS0EsWUFBMUI7QUFDQSxXQUFLdEIsTUFBTCxDQUFZQyxPQUFaLENBQW9CO0FBQ2xCaEIsUUFBQUEsSUFBSSxFQUFFLFlBRFk7QUFFbEJDLFFBQUFBLEtBQUssRUFBRSxhQUZXO0FBR2xCZ0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHUDtBQUNUQSxVQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELFVBQUFBLENBQUMsQ0FBQ0csUUFBRixHQUFhLElBQWI7QUFDQUgsVUFBQUEsQ0FBQyxDQUFDRSxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBUGlCLE9BQXBCO0FBU0EsV0FBS0wsTUFBTCxDQUFZQyxPQUFaLENBQW9CO0FBQ2xCaEIsUUFBQUEsSUFBSSxFQUFFLFlBRFk7QUFFbEJDLFFBQUFBLEtBQUssRUFBRSxlQUZXO0FBR2xCZ0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHUDtBQUNUQSxVQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELFVBQUFBLENBQUMsQ0FBQ0UsUUFBRixHQUFhLElBQWI7QUFDRDtBQU5pQixPQUFwQjtBQVFBLFdBQUtMLE1BQUwsQ0FBWUMsT0FBWixDQUFvQjtBQUNsQmhCLFFBQUFBLElBQUksRUFBRSxhQURZO0FBRWxCQyxRQUFBQSxLQUFLLEVBQUUsY0FGVztBQUdsQmdCLFFBQUFBLE9BSGtCLG1CQUdWQyxDQUhVLEVBR1A7QUFDVEEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNFLFFBQUYsR0FBYSxJQUFiO0FBQ0Q7QUFOaUIsT0FBcEI7QUFRQSxXQUFLTCxNQUFMLENBQVlDLE9BQVosQ0FBb0I7QUFDbEJoQixRQUFBQSxJQUFJLEVBQUUsV0FEWTtBQUVsQkMsUUFBQUEsS0FBSyxFQUFFLFlBRlc7QUFHbEJnQixRQUFBQSxPQUhrQixtQkFHVkMsQ0FIVSxFQUdQO0FBQ1RBLFVBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsVUFBQUEsQ0FBQyxDQUFDRSxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBTmlCLE9BQXBCO0FBUUEsV0FBS0wsTUFBTCxDQUFZc0MsT0FBWixDQUFvQjtBQUNsQnJELFFBQUFBLElBQUksRUFBRSxTQURZO0FBRWxCQyxRQUFBQSxLQUFLLEVBQUUsU0FGVztBQUdsQmdCLFFBQUFBLE9BSGtCLG1CQUdWQyxDQUhVLEVBR1A7QUFDVEEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNFLFFBQUYsR0FBYSxJQUFiO0FBQ0Q7QUFOaUIsT0FBcEI7QUFRQSxXQUFLTCxNQUFMLENBQVlzQyxPQUFaLENBQW9CO0FBQ2xCckQsUUFBQUEsSUFBSSxFQUFFLFdBRFk7QUFFbEJDLFFBQUFBLEtBQUssRUFBRSxXQUZXO0FBR2xCZ0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHUDtBQUNUQSxVQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELFVBQUFBLENBQUMsQ0FBQ0UsUUFBRixHQUFhLElBQWI7QUFDRDtBQU5pQixPQUFwQjtBQVFBLFdBQUtMLE1BQUwsQ0FBWUMsT0FBWixDQUFvQjtBQUNsQmhCLFFBQUFBLElBQUksRUFBRSxRQURZO0FBRWxCQyxRQUFBQSxLQUFLLEVBQUUsU0FGVztBQUdsQmdCLFFBQUFBLE9BSGtCLG1CQUdWQyxDQUhVLEVBR1A7QUFDVEEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNFLFFBQUYsR0FBYSxJQUFiO0FBQ0Q7QUFOaUIsT0FBcEI7QUFRQSxXQUFLTCxNQUFMLENBQVlDLE9BQVosQ0FBb0I7QUFDbEJoQixRQUFBQSxJQUFJLEVBQUUsYUFEWTtBQUVsQkMsUUFBQUEsS0FBSyxFQUFFLGNBRlc7QUFHbEJnQixRQUFBQSxPQUhrQixtQkFHVkMsQ0FIVSxFQUdQO0FBQ1RBLFVBQUFBLENBQUMsQ0FBQ2lCLFFBQUYsR0FBYSxJQUFiO0FBQ0FqQixVQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0Q7QUFOaUIsT0FBcEI7QUFRQSxXQUFLSixNQUFMLENBQVlDLE9BQVosQ0FBb0I7QUFDbEJoQixRQUFBQSxJQUFJLEVBQUUsWUFEWTtBQUVsQkMsUUFBQUEsS0FBSyxFQUFFLGFBRlc7QUFHbEJnQixRQUFBQSxPQUhrQixtQkFHVkMsQ0FIVSxFQUdQO0FBQ1RBLFVBQUFBLENBQUMsQ0FBQ2lCLFFBQUYsR0FBYSxJQUFiO0FBQ0FqQixVQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0Q7QUFOaUIsT0FBcEI7QUFRQSxXQUFLSixNQUFMLENBQVlDLE9BQVosQ0FBb0I7QUFDbEJoQixRQUFBQSxJQUFJLEVBQUUsY0FEWTtBQUVsQkMsUUFBQUEsS0FBSyxFQUFFLGVBRlc7QUFHbEJnQixRQUFBQSxPQUhrQixtQkFHVkMsQ0FIVSxFQUdQO0FBQ1RBLFVBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDRDtBQUxpQixPQUFwQjtBQU9BLFdBQUtKLE1BQUwsQ0FBWVEsT0FBWixDQUFvQjtBQUNsQnZCLFFBQUFBLElBQUksRUFBRSxnQkFEWTtBQUVsQkMsUUFBQUEsS0FBSyxFQUFFLGlCQUZXO0FBR2xCZ0IsUUFBQUEsT0FIa0IsbUJBR1ZDLENBSFUsRUFHRztBQUNuQkEsVUFBQUEsQ0FBQyxDQUFDQyxTQUFGLEdBQWMsSUFBZDtBQUNBRCxVQUFBQSxDQUFDLENBQUNNLE1BQUYsR0FBVyxJQUFYO0FBQ0FOLFVBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixZQUFBQSxRQUFRLFlBQUsyQyxZQUFMO0FBREMsV0FBWDtBQUdEO0FBVGlCLE9BQXBCO0FBV0EsV0FBS3RCLE1BQUwsQ0FBWVEsT0FBWixDQUFvQjtBQUNsQnZCLFFBQUFBLElBQUksRUFBRSxhQURZO0FBRWxCQyxRQUFBQSxLQUFLLEVBQUUsY0FGVztBQUdsQmdCLFFBQUFBLE9BSGtCLG1CQUdWQyxDQUhVLEVBR0c7QUFDbkJBLFVBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsVUFBQUEsQ0FBQyxDQUFDRyxRQUFGLEdBQWEsSUFBYjtBQUNBSCxVQUFBQSxDQUFDLENBQUNNLE1BQUYsR0FBVyxJQUFYO0FBQ0FOLFVBQUFBLENBQUMsQ0FBQ08sTUFBRixHQUFXO0FBQ1QvQixZQUFBQSxRQUFRLFlBQUsyQyxZQUFMO0FBREMsV0FBWDtBQUdEO0FBVmlCLE9BQXBCO0FBWUEsV0FBS3RCLE1BQUwsQ0FBWVEsT0FBWixDQUFvQjtBQUNsQnZCLFFBQUFBLElBQUksRUFBRSxjQURZO0FBRWxCQyxRQUFBQSxLQUFLLEVBQUUsZUFGVztBQUdsQmdCLFFBQUFBLE9BSGtCLG1CQUdWQyxDQUhVLEVBR0c7QUFDbkJBLFVBQUFBLENBQUMsQ0FBQ0MsU0FBRixHQUFjLElBQWQ7QUFDQUQsVUFBQUEsQ0FBQyxDQUFDTSxNQUFGLEdBQVcsSUFBWDtBQUNBTixVQUFBQSxDQUFDLENBQUNPLE1BQUYsR0FBVztBQUNUL0IsWUFBQUEsUUFBUSxZQUFLMkMsWUFBTDtBQURDLFdBQVg7QUFHRDtBQVRpQixPQUFwQjtBQVdBLFdBQUt0QixNQUFMLENBQVlRLE9BQVosQ0FBb0I7QUFDbEJ2QixRQUFBQSxJQUFJLEVBQUUsY0FEWTtBQUVsQkMsUUFBQUEsS0FBSyxFQUFFLGVBRlc7QUFHbEJnQixRQUFBQSxPQUhrQixtQkFHVkMsQ0FIVSxFQUdHO0FBQ25CQSxVQUFBQSxDQUFDLENBQUNDLFNBQUYsR0FBYyxJQUFkO0FBQ0FELFVBQUFBLENBQUMsQ0FBQ00sTUFBRixHQUFXLElBQVg7QUFDQU4sVUFBQUEsQ0FBQyxDQUFDTyxNQUFGLEdBQVc7QUFDVC9CLFlBQUFBLFFBQVEsRUFBRTtBQURELFdBQVg7QUFHRDtBQVRpQixPQUFwQjtBQVlBLFdBQUsrQyxhQUFMO0FBQ0Q7OzsyQkFFYztBQUNiLGFBQU8sbUJBQVA7QUFDRDs7O0VBcEpvQzdCLFk7Ozs7SUErSjFCMEMsd0I7QUFLWCxvQ0FBWXpDLElBQVosRUFBdUQ7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUNyRCxTQUFLMEMsU0FBTCxHQUFpQixJQUFJbkIsZUFBSixDQUFvQjtBQUNuQzFDLE1BQUFBLFFBQVEsRUFBRW1CLElBQUksQ0FBQ25CLFFBRG9CO0FBRW5DQyxNQUFBQSxlQUFlLEVBQUVrQixJQUFJLENBQUNsQixlQUZhO0FBR25DRSxNQUFBQSxVQUFVLEVBQUVnQixJQUFJLENBQUNoQixVQUhrQjtBQUluQ0QsTUFBQUEsV0FBVyxFQUFFaUIsSUFBSSxDQUFDakI7QUFKaUIsS0FBcEIsQ0FBakI7QUFNQSxTQUFLNEQsTUFBTCxHQUFjLElBQUlWLFlBQUosQ0FBaUI7QUFDN0JwRCxNQUFBQSxRQUFRLEVBQUVtQixJQUFJLENBQUNuQixRQURjO0FBRTdCQyxNQUFBQSxlQUFlLEVBQUVrQixJQUFJLENBQUNsQixlQUZPO0FBRzdCRSxNQUFBQSxVQUFVLEVBQUVnQixJQUFJLENBQUNoQixVQUhZO0FBSTdCRCxNQUFBQSxXQUFXLEVBQUVpQixJQUFJLENBQUNqQjtBQUpXLEtBQWpCLENBQWQ7QUFNQSxTQUFLNkQsV0FBTCxHQUFtQixJQUFJTixpQkFBSixDQUFzQjtBQUN2Q3pELE1BQUFBLFFBQVEsRUFBRW1CLElBQUksQ0FBQ25CLFFBRHdCO0FBRXZDQyxNQUFBQSxlQUFlLEVBQUVrQixJQUFJLENBQUNsQixlQUZpQjtBQUd2Q0UsTUFBQUEsVUFBVSxFQUFFZ0IsSUFBSSxDQUFDaEIsVUFIc0I7QUFJdkNELE1BQUFBLFdBQVcsRUFBRWlCLElBQUksQ0FBQ2pCO0FBSnFCLEtBQXRCLENBQW5CO0FBTUQ7Ozs7d0JBRXdEO0FBQ3ZELFVBQU1zRCxJQUFJLEdBQUcsS0FBS00sTUFBTCxDQUFZekMsTUFBWixDQUFtQjhCLFFBQW5CLENBQTRCLFlBQTVCLENBQWI7QUFDQUssTUFBQUEsSUFBSSxDQUFDeEMsVUFBTCxDQUFnQmdELGVBQWhCLEdBQWtDLElBQWxDO0FBQ0EsYUFBT1IsSUFBSSxDQUFDeEMsVUFBWjtBQUNEOzs7d0JBRTREO0FBQzNELFVBQU13QyxJQUFJLEdBQUcsS0FBS0ssU0FBTCxDQUFleEMsTUFBZixDQUFzQjhCLFFBQXRCLENBQStCLGFBQS9CLENBQWI7QUFDQSxhQUFPSyxJQUFJLENBQUN4QyxVQUFaO0FBQ0QiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcm9wTGluayB9IGZyb20gXCIuL3Byb3AvbGlua1wiO1xuaW1wb3J0IHsgUHJvcE51bWJlciB9IGZyb20gXCIuL3Byb3AvbnVtYmVyXCI7XG5pbXBvcnQge1xuICBQcm9wT2JqZWN0LFxuICBQcm9wTWV0aG9kLFxuICBQcm9wQWN0aW9uLFxuICBJbnRlZ3JhdGlvblNlcnZpY2UsXG59IGZyb20gXCIuL2F0dHJMaXN0XCI7XG5pbXBvcnQgeyBjYW1lbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCB7IEFzc29jaWF0aW9uTGlzdCB9IGZyb20gXCIuL3N5c3RlbU9iamVjdC9hc3NvY2lhdGlvbnNcIjtcbmltcG9ydCB7IFNpR3JhcGhxbCB9IGZyb20gXCIuL3N5c3RlbU9iamVjdC9ncmFwaHFsXCI7XG5cbmV4cG9ydCB0eXBlIE9iamVjdFR5cGVzID1cbiAgfCBCYXNlT2JqZWN0XG4gIHwgU3lzdGVtT2JqZWN0XG4gIHwgQ29tcG9uZW50T2JqZWN0XG4gIHwgRW50aXR5T2JqZWN0XG4gIHwgRW50aXR5RXZlbnRPYmplY3Q7XG5cbmV4cG9ydCBpbnRlcmZhY2UgQmFzZU9iamVjdENvbnN0cnVjdG9yIHtcbiAgdHlwZU5hbWU6IEJhc2VPYmplY3RbXCJ0eXBlTmFtZVwiXTtcbiAgZGlzcGxheVR5cGVOYW1lOiBCYXNlT2JqZWN0W1wiZGlzcGxheVR5cGVOYW1lXCJdO1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzaVBhdGhOYW1lPzogc3RyaW5nO1xuICBvcHRpb25zPyhjOiBCYXNlT2JqZWN0KTogdm9pZDtcbn1cblxuZXhwb3J0IGludGVyZmFjZSBBZGRNZXRob2RDb25zdHJ1Y3RvciB7XG4gIGlzUHJpdmF0ZT86IFByb3BNZXRob2RbXCJpc1ByaXZhdGVcIl07XG59XG5cbmV4cG9ydCBjbGFzcyBCYXNlT2JqZWN0IHtcbiAgdHlwZU5hbWU6IHN0cmluZztcbiAgZGlzcGxheVR5cGVOYW1lOiBzdHJpbmc7XG4gIHNpUGF0aE5hbWU6IHN0cmluZztcbiAgc2VydmljZU5hbWU6IHN0cmluZztcbiAgbXZjYzogYm9vbGVhbjtcblxuICByb290UHJvcDogUHJvcE9iamVjdDtcbiAgbWV0aG9kc1Byb3A6IFByb3BPYmplY3Q7XG4gIGFzc29jaWF0aW9uczogQXNzb2NpYXRpb25MaXN0O1xuXG4gIHByaXZhdGUgaW50ZXJuYWxHcmFwaHFsOiB1bmRlZmluZWQgfCBTaUdyYXBocWw7XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIHR5cGVOYW1lLFxuICAgIGRpc3BsYXlUeXBlTmFtZSxcbiAgICBzZXJ2aWNlTmFtZSxcbiAgICBzaVBhdGhOYW1lID0gXCJcIixcbiAgfTogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgdGhpcy50eXBlTmFtZSA9IGNhbWVsQ2FzZSh0eXBlTmFtZSk7XG4gICAgdGhpcy5kaXNwbGF5VHlwZU5hbWUgPSBkaXNwbGF5VHlwZU5hbWU7XG4gICAgdGhpcy5zaVBhdGhOYW1lID0gc2lQYXRoTmFtZTtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWUgfHwgdHlwZU5hbWU7XG4gICAgdGhpcy5yb290UHJvcCA9IG5ldyBQcm9wT2JqZWN0KHtcbiAgICAgIG5hbWU6IHR5cGVOYW1lLFxuICAgICAgbGFiZWw6IGRpc3BsYXlUeXBlTmFtZSxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0eXBlTmFtZSxcbiAgICAgIHBhcmVudE5hbWU6IFwiXCIsXG4gICAgfSk7XG4gICAgdGhpcy5tZXRob2RzUHJvcCA9IG5ldyBQcm9wT2JqZWN0KHtcbiAgICAgIG5hbWU6IGAke3R5cGVOYW1lfWAsXG4gICAgICBsYWJlbDogYCR7ZGlzcGxheVR5cGVOYW1lfSBNZXRob2RzYCxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0eXBlTmFtZSxcbiAgICAgIHBhcmVudE5hbWU6IFwiXCIsXG4gICAgfSk7XG4gICAgdGhpcy5hc3NvY2lhdGlvbnMgPSBuZXcgQXNzb2NpYXRpb25MaXN0KCk7XG4gICAgdGhpcy5pbnRlcm5hbEdyYXBocWwgPSB1bmRlZmluZWQ7XG4gICAgdGhpcy5tdmNjID0gZmFsc2U7XG4gIH1cblxuICBnZXQgZmllbGRzKCk6IEJhc2VPYmplY3RbXCJyb290UHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIHJldHVybiB0aGlzLnJvb3RQcm9wLnByb3BlcnRpZXM7XG4gIH1cblxuICBnZXQgbWV0aG9kcygpOiBCYXNlT2JqZWN0W1wibWV0aG9kc1Byb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICByZXR1cm4gdGhpcy5tZXRob2RzUHJvcC5wcm9wZXJ0aWVzO1xuICB9XG5cbiAgZ2V0IGdyYXBocWwoKTogU2lHcmFwaHFsIHtcbiAgICBpZiAodGhpcy5pbnRlcm5hbEdyYXBocWwgPT0gdW5kZWZpbmVkKSB7XG4gICAgICB0aGlzLmludGVybmFsR3JhcGhxbCA9IG5ldyBTaUdyYXBocWwodGhpcyk7XG4gICAgfVxuICAgIHJldHVybiB0aGlzLmludGVybmFsR3JhcGhxbDtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJiYXNlT2JqZWN0XCI7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFN5c3RlbU9iamVjdCBleHRlbmRzIEJhc2VPYmplY3Qge1xuICBuYXR1cmFsS2V5ID0gXCJuYW1lXCI7XG4gIG1pZ3JhdGVhYmxlID0gZmFsc2U7XG5cbiAgY29uc3RydWN0b3IoYXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgc3VwZXIoYXJncyk7XG4gICAgdGhpcy5zZXRTeXN0ZW1PYmplY3REZWZhdWx0cygpO1xuICB9XG5cbiAgc2V0U3lzdGVtT2JqZWN0RGVmYXVsdHMoKTogdm9pZCB7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImlkXCIsXG4gICAgICBsYWJlbDogYCR7dGhpcy5kaXNwbGF5VHlwZU5hbWV9IElEYCxcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgaWYgKCF0aGlzLnR5cGVOYW1lLmVuZHNXaXRoKFwiRW50aXR5RXZlbnRcIikpIHtcbiAgICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgICBuYW1lOiBcIm5hbWVcIixcbiAgICAgICAgbGFiZWw6IGAke3RoaXMuZGlzcGxheVR5cGVOYW1lfSBOYW1lYCxcbiAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICB9LFxuICAgICAgfSk7XG4gICAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgICAgbmFtZTogXCJkaXNwbGF5TmFtZVwiLFxuICAgICAgICBsYWJlbDogYCR7dGhpcy5kaXNwbGF5VHlwZU5hbWV9IERpc3BsYXkgTmFtZWAsXG4gICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgfSxcbiAgICAgIH0pO1xuICAgIH1cbiAgICB0aGlzLmZpZWxkcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwic2lTdG9yYWJsZVwiLFxuICAgICAgbGFiZWw6IFwiU0kgU3RvcmFibGVcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBcImRhdGFTdG9yYWJsZVwiLFxuICAgICAgICB9O1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwic3lzdGVtT2JqZWN0XCI7XG4gIH1cblxuICBhZGRHZXRNZXRob2QoYXJnczogQWRkTWV0aG9kQ29uc3RydWN0b3IgPSB7fSk6IHZvaWQge1xuICAgIC8vIGVzbGludC1kaXNhYmxlLW5leHQtbGluZVxuICAgIGNvbnN0IHN5c3RlbU9iamVjdCA9IHRoaXM7XG5cbiAgICBzeXN0ZW1PYmplY3QubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJnZXRcIixcbiAgICAgIGxhYmVsOiBgR2V0IGEgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfWAsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5pc1ByaXZhdGUgPSBhcmdzLmlzUHJpdmF0ZSB8fCBmYWxzZTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJpZFwiLFxuICAgICAgICAgIGxhYmVsOiBgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfSBJRGAsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiaXRlbVwiLFxuICAgICAgICAgIGxhYmVsOiBgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfSBJdGVtYCxcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IHN5c3RlbU9iamVjdC50eXBlTmFtZSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAgYWRkTGlzdE1ldGhvZChhcmdzOiBBZGRNZXRob2RDb25zdHJ1Y3RvciA9IHt9KTogdm9pZCB7XG4gICAgLy8gZXNsaW50LWRpc2FibGUtbmV4dC1saW5lXG4gICAgY29uc3Qgc3lzdGVtT2JqZWN0ID0gdGhpcztcbiAgICBzeXN0ZW1PYmplY3QubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJsaXN0XCIsXG4gICAgICBsYWJlbDogYExpc3QgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfWAsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmlzUHJpdmF0ZSA9IGFyZ3MuaXNQcml2YXRlIHx8IGZhbHNlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInF1ZXJ5XCIsXG4gICAgICAgICAgbGFiZWw6IFwiUXVlcnlcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IFwiZGF0YVF1ZXJ5XCIsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGROdW1iZXIoe1xuICAgICAgICAgIG5hbWU6IFwicGFnZVNpemVcIixcbiAgICAgICAgICBsYWJlbDogXCJQYWdlIFNpemVcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BOdW1iZXIpIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubnVtYmVyS2luZCA9IFwidWludDMyXCI7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwib3JkZXJCeVwiLFxuICAgICAgICAgIGxhYmVsOiBcIk9yZGVyIEJ5XCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwib3JkZXJCeURpcmVjdGlvblwiLFxuICAgICAgICAgIGxhYmVsOiBcIk9yZGVyIEJ5IERpcmVjdGlvblwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogXCJkYXRhUGFnZVRva2VuXCIsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJvcmRlckJ5RGlyZWN0aW9uXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJwYWdlVG9rZW5cIixcbiAgICAgICAgICBsYWJlbDogXCJQYWdlIFRva2VuXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwic2NvcGVCeVRlbmFudElkXCIsXG4gICAgICAgICAgbGFiZWw6IFwiU2NvcGUgQnkgVGVuYW50IElEXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcIml0ZW1zXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSXRlbXNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVwZWF0ZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBzeXN0ZW1PYmplY3QudHlwZU5hbWUsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTnVtYmVyKHtcbiAgICAgICAgICBuYW1lOiBcInRvdGFsQ291bnRcIixcbiAgICAgICAgICBsYWJlbDogXCJUb3RhbCBDb3VudFwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcE51bWJlcikge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5udW1iZXJLaW5kID0gXCJ1aW50MzJcIjtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwibmV4dFBhZ2VUb2tlblwiLFxuICAgICAgICAgIGxhYmVsOiBcIk5leHQgUGFnZSBUb2tlblwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgQ29tcG9uZW50T2JqZWN0IGV4dGVuZHMgU3lzdGVtT2JqZWN0IHtcbiAgYmFzZVR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3IoYXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgY29uc3QgdHlwZU5hbWUgPSBgJHthcmdzLnR5cGVOYW1lfUNvbXBvbmVudGA7XG4gICAgY29uc3QgZGlzcGxheVR5cGVOYW1lID0gYCR7YXJncy5kaXNwbGF5VHlwZU5hbWV9IENvbXBvbmVudGA7XG4gICAgc3VwZXIoe1xuICAgICAgdHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmJhc2VUeXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5zZXRDb21wb25lbnREZWZhdWx0cygpO1xuICB9XG5cbiAgc2V0Q29tcG9uZW50RGVmYXVsdHMoKTogdm9pZCB7XG4gICAgY29uc3QgYmFzZVR5cGVOYW1lID0gdGhpcy5iYXNlVHlwZU5hbWU7XG5cbiAgICB0aGlzLm1pZ3JhdGVhYmxlID0gdHJ1ZTtcblxuICAgIHRoaXMuYWRkR2V0TWV0aG9kKCk7XG4gICAgdGhpcy5hZGRMaXN0TWV0aG9kKCk7XG5cbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgIGxhYmVsOiBcIkNvbXBvbmVudCBEZXNjcmlwdGlvblwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZE9iamVjdCh7XG4gICAgICBuYW1lOiBcImNvbnN0cmFpbnRzXCIsXG4gICAgICBsYWJlbDogXCJDb21wb25lbnQgQ29uc3RyYWludHNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE9iamVjdCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICBwLnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJjb21wb25lbnROYW1lXCIsXG4gICAgICAgICAgbGFiZWw6IFwiQ29tcG9uZW50IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiY29tcG9uZW50RGlzcGxheU5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJDb21wb25lbnQgRGlzcGxheSBOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJzaVByb3BlcnRpZXNcIixcbiAgICAgIGxhYmVsOiBcIlNJIFByb3BlcnRpZXNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogXCJjb21wb25lbnRTaVByb3BlcnRpZXNcIixcbiAgICAgICAgfTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImNyZWF0ZVwiLFxuICAgICAgbGFiZWw6IFwiQ3JlYXRlIGEgQ29tcG9uZW50XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5tdXRhdGlvbiA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5pc1ByaXZhdGUgPSB0cnVlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcIm5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJJbnRlZ3JhdGlvbiBOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJkaXNwbGF5TmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkludGVncmF0aW9uIERpc3BsYXkgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgICAgICBsYWJlbDogXCJJbnRlZ3JhdGlvbiBEaXNwbGF5IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImNvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiQ29uc3RyYWludHNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgICAgIGxhYmVsOiBcIlNpIFByb3BlcnRpZXNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogXCJjb21wb25lbnRTaVByb3BlcnRpZXNcIixcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcIml0ZW1cIixcbiAgICAgICAgICBsYWJlbDogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudCBJdGVtYCxcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLm1ldGhvZHMuYWRkTWV0aG9kKHtcbiAgICAgIG5hbWU6IFwicGlja1wiLFxuICAgICAgbGFiZWw6IFwiUGljayBDb21wb25lbnRcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImNvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiQ29uc3RyYWludHNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpbXBsaWNpdENvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSW1wbGljaXQgQ29uc3RyYWludHNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImNvbXBvbmVudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIkNob3NlbiBDb21wb25lbnRcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cblxuICBnZXQgY29uc3RyYWludHMoKTogQ29tcG9uZW50T2JqZWN0W1wicm9vdFByb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICBjb25zdCBjb25zdHJhaW50UHJvcCA9IHRoaXMuZmllbGRzLmdldEVudHJ5KFwiY29uc3RyYWludHNcIikgYXMgUHJvcE9iamVjdDtcbiAgICByZXR1cm4gY29uc3RyYWludFByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJjb21wb25lbnRPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgRW50aXR5T2JqZWN0IGV4dGVuZHMgU3lzdGVtT2JqZWN0IHtcbiAgYmFzZVR5cGVOYW1lOiBzdHJpbmc7XG4gIGludGVncmF0aW9uU2VydmljZXM6IEludGVncmF0aW9uU2VydmljZVtdO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIGNvbnN0IHR5cGVOYW1lID0gYCR7YXJncy50eXBlTmFtZX1FbnRpdHlgO1xuICAgIGNvbnN0IGRpc3BsYXlUeXBlTmFtZSA9IGAke2FyZ3MuZGlzcGxheVR5cGVOYW1lfSBFbnRpdHlgO1xuICAgIHN1cGVyKHtcbiAgICAgIHR5cGVOYW1lLFxuICAgICAgZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5iYXNlVHlwZU5hbWUgPSBhcmdzLnR5cGVOYW1lO1xuICAgIHRoaXMuaW50ZWdyYXRpb25TZXJ2aWNlcyA9IFtdO1xuICAgIHRoaXMuc2V0RW50aXR5RGVmYXVsdHMoKTtcbiAgfVxuXG4gIHNldEVudGl0eURlZmF1bHRzKCk6IHZvaWQge1xuICAgIGNvbnN0IGJhc2VUeXBlTmFtZSA9IHRoaXMuYmFzZVR5cGVOYW1lO1xuXG4gICAgdGhpcy5tdmNjID0gdHJ1ZTtcblxuICAgIHRoaXMuYWRkR2V0TWV0aG9kKCk7XG4gICAgdGhpcy5hZGRMaXN0TWV0aG9kKCk7XG5cbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgIGxhYmVsOiBcIkVudGl0eSBEZXNjcmlwdGlvblwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJzaVByb3BlcnRpZXNcIixcbiAgICAgIGxhYmVsOiBcIlNJIFByb3BlcnRpZXNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogXCJlbnRpdHlTaVByb3BlcnRpZXNcIixcbiAgICAgICAgfTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZE9iamVjdCh7XG4gICAgICBuYW1lOiBcInByb3BlcnRpZXNcIixcbiAgICAgIGxhYmVsOiBcIlByb3BlcnRpZXNcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgIGxhYmVsOiBcIkNvbnN0cmFpbnRzXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJpbXBsaWNpdENvbnN0cmFpbnRzXCIsXG4gICAgICBsYWJlbDogXCJJbXBsaWNpdCBDb25zdHJhaW50c1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgbmFtZXM6IFtcImNvbnN0cmFpbnRzXCJdLFxuICAgICAgICB9O1xuICAgICAgfSxcbiAgICB9KTtcblxuICAgIHRoaXMubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJjcmVhdGVcIixcbiAgICAgIGxhYmVsOiBcIkNyZWF0ZSBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLm11dGF0aW9uID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJuYW1lXCIsXG4gICAgICAgICAgbGFiZWw6IFwiTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiZGlzcGxheU5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJEaXNwbGF5IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImRlc2NyaXB0aW9uXCIsXG4gICAgICAgICAgbGFiZWw6IFwiRGVzY3JpcHRpb25cIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcIndvcmtzcGFjZUlkXCIsXG4gICAgICAgICAgbGFiZWw6IGBXb3Jrc3BhY2UgSURgLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiY2hhbmdlU2V0SWRcIixcbiAgICAgICAgICBsYWJlbDogYENoYW5nZSBTZXQgSURgLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInByb3BlcnRpZXNcIixcbiAgICAgICAgICBsYWJlbDogXCJQcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJwcm9wZXJ0aWVzXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJjb25zdHJhaW50c1wiLFxuICAgICAgICAgIGxhYmVsOiBcIkNvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpdGVtXCIsXG4gICAgICAgICAgbGFiZWw6IFwiJHtiYXNlVHlwZU5hbWV9RW50aXR5IEl0ZW1cIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJlbnRpdHlFdmVudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIkVudGl0eSBFdmVudFwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5RXZlbnRgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkQWN0aW9uKHtcbiAgICAgIG5hbWU6IFwic3luY1wiLFxuICAgICAgbGFiZWw6IFwiU3luYyBTdGF0ZVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wQWN0aW9uKSB7XG4gICAgICAgIHAubXV0YXRpb24gPSB0cnVlO1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAgZ2V0IHByb3BlcnRpZXMoKTogRW50aXR5T2JqZWN0W1wicm9vdFByb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICBjb25zdCBwcm9wID0gdGhpcy5maWVsZHMuZ2V0RW50cnkoXCJwcm9wZXJ0aWVzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJlbnRpdHlPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgRW50aXR5RXZlbnRPYmplY3QgZXh0ZW5kcyBTeXN0ZW1PYmplY3Qge1xuICBiYXNlVHlwZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICBjb25zdCB0eXBlTmFtZSA9IGAke2FyZ3MudHlwZU5hbWV9RW50aXR5RXZlbnRgO1xuICAgIGNvbnN0IGRpc3BsYXlUeXBlTmFtZSA9IGAke2FyZ3MuZGlzcGxheVR5cGVOYW1lfSBFbnRpdHlFdmVudGA7XG4gICAgc3VwZXIoe1xuICAgICAgdHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmJhc2VUeXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5zZXRFbnRpdHlFdmVudERlZmF1bHRzKCk7XG4gIH1cblxuICBzZXRFbnRpdHlFdmVudERlZmF1bHRzKCk6IHZvaWQge1xuICAgIGNvbnN0IGJhc2VUeXBlTmFtZSA9IHRoaXMuYmFzZVR5cGVOYW1lO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJhY3Rpb25OYW1lXCIsXG4gICAgICBsYWJlbDogXCJBY3Rpb24gTmFtZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiY3JlYXRlVGltZVwiLFxuICAgICAgbGFiZWw6IFwiQ3JlYXRpb24gVGltZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJ1cGRhdGVkVGltZVwiLFxuICAgICAgbGFiZWw6IFwiVXBkYXRlZCBUaW1lXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImZpbmFsVGltZVwiLFxuICAgICAgbGFiZWw6IFwiRmluYWwgVGltZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZEJvb2woe1xuICAgICAgbmFtZTogXCJzdWNjZXNzXCIsXG4gICAgICBsYWJlbDogXCJzdWNjZXNzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkQm9vbCh7XG4gICAgICBuYW1lOiBcImZpbmFsaXplZFwiLFxuICAgICAgbGFiZWw6IFwiRmluYWxpemVkXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcInVzZXJJZFwiLFxuICAgICAgbGFiZWw6IFwiVXNlciBJRFwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJvdXRwdXRMaW5lc1wiLFxuICAgICAgbGFiZWw6IFwiT3V0cHV0IExpbmVzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC5yZXBlYXRlZCA9IHRydWU7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImVycm9yTGluZXNcIixcbiAgICAgIGxhYmVsOiBcIkVycm9yIExpbmVzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC5yZXBlYXRlZCA9IHRydWU7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImVycm9yTWVzc2FnZVwiLFxuICAgICAgbGFiZWw6IFwiRXJyb3IgTWVzc2FnZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInByZXZpb3VzRW50aXR5XCIsXG4gICAgICBsYWJlbDogXCJQcmV2aW91cyBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcImlucHV0RW50aXR5XCIsXG4gICAgICBsYWJlbDogXCJJbnB1dCBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJvdXRwdXRFbnRpdHlcIixcbiAgICAgIGxhYmVsOiBcIk91dHB1dCBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IFwiZW50aXR5RXZlbnRTaVByb3BlcnRpZXNcIixcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLmFkZExpc3RNZXRob2QoKTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJlbnRpdHlFdmVudE9iamVjdFwiO1xuICB9XG59XG5cbmV4cG9ydCBpbnRlcmZhY2UgQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0Q29uc3RydWN0b3Ige1xuICB0eXBlTmFtZTogQmFzZU9iamVjdFtcInR5cGVOYW1lXCJdO1xuICBkaXNwbGF5VHlwZU5hbWU6IEJhc2VPYmplY3RbXCJkaXNwbGF5VHlwZU5hbWVcIl07XG4gIHNpUGF0aE5hbWU/OiBzdHJpbmc7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIG9wdGlvbnM/KGM6IENvbXBvbmVudEFuZEVudGl0eU9iamVjdCk6IHZvaWQ7XG59XG5cbmV4cG9ydCBjbGFzcyBDb21wb25lbnRBbmRFbnRpdHlPYmplY3Qge1xuICBjb21wb25lbnQ6IENvbXBvbmVudE9iamVjdDtcbiAgZW50aXR5OiBFbnRpdHlPYmplY3Q7XG4gIGVudGl0eUV2ZW50OiBFbnRpdHlFdmVudE9iamVjdDtcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBDb21wb25lbnRBbmRFbnRpdHlPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIHRoaXMuY29tcG9uZW50ID0gbmV3IENvbXBvbmVudE9iamVjdCh7XG4gICAgICB0eXBlTmFtZTogYXJncy50eXBlTmFtZSxcbiAgICAgIGRpc3BsYXlUeXBlTmFtZTogYXJncy5kaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzaVBhdGhOYW1lOiBhcmdzLnNpUGF0aE5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmVudGl0eSA9IG5ldyBFbnRpdHlPYmplY3Qoe1xuICAgICAgdHlwZU5hbWU6IGFyZ3MudHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWU6IGFyZ3MuZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2lQYXRoTmFtZTogYXJncy5zaVBhdGhOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5lbnRpdHlFdmVudCA9IG5ldyBFbnRpdHlFdmVudE9iamVjdCh7XG4gICAgICB0eXBlTmFtZTogYXJncy50eXBlTmFtZSxcbiAgICAgIGRpc3BsYXlUeXBlTmFtZTogYXJncy5kaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzaVBhdGhOYW1lOiBhcmdzLnNpUGF0aE5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgfVxuXG4gIGdldCBwcm9wZXJ0aWVzKCk6IEVudGl0eU9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgY29uc3QgcHJvcCA9IHRoaXMuZW50aXR5LmZpZWxkcy5nZXRFbnRyeShcInByb3BlcnRpZXNcIikgYXMgUHJvcE9iamVjdDtcbiAgICBwcm9wLnByb3BlcnRpZXMuYXV0b0NyZWF0ZUVkaXRzID0gdHJ1ZTtcbiAgICByZXR1cm4gcHJvcC5wcm9wZXJ0aWVzO1xuICB9XG5cbiAgZ2V0IGNvbnN0cmFpbnRzKCk6IENvbXBvbmVudE9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgY29uc3QgcHJvcCA9IHRoaXMuY29tcG9uZW50LmZpZWxkcy5nZXRFbnRyeShcImNvbnN0cmFpbnRzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxufVxuIl19