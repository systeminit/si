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
    _this3.baseTypeName = args.typeName;

    _this3.setEntityDefaults();

    return _this3;
  }

  (0, _createClass2["default"])(EntityObject, [{
    key: "setEntityDefaults",
    value: function setEntityDefaults() {
      var baseTypeName = this.baseTypeName;
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
          p.request.properties.addLink({
            name: "siProperties",
            label: "Si Properties",
            options: function options(p) {
              p.required = true;
              p.lookup = {
                typeName: "entitySiProperties"
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9zeXN0ZW1Db21wb25lbnQudHMiXSwibmFtZXMiOlsiQmFzZU9iamVjdCIsInR5cGVOYW1lIiwiZGlzcGxheVR5cGVOYW1lIiwic2VydmljZU5hbWUiLCJzaVBhdGhOYW1lIiwicm9vdFByb3AiLCJQcm9wT2JqZWN0IiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJwYXJlbnROYW1lIiwibWV0aG9kc1Byb3AiLCJhc3NvY2lhdGlvbnMiLCJBc3NvY2lhdGlvbkxpc3QiLCJpbnRlcm5hbEdyYXBocWwiLCJ1bmRlZmluZWQiLCJwcm9wZXJ0aWVzIiwiU2lHcmFwaHFsIiwiU3lzdGVtT2JqZWN0IiwiYXJncyIsInNldFN5c3RlbU9iamVjdERlZmF1bHRzIiwiZmllbGRzIiwiYWRkVGV4dCIsIm9wdGlvbnMiLCJwIiwidW5pdmVyc2FsIiwicmVhZE9ubHkiLCJyZXF1aXJlZCIsImFkZExpbmsiLCJoaWRkZW4iLCJsb29rdXAiLCJzeXN0ZW1PYmplY3QiLCJtZXRob2RzIiwiYWRkTWV0aG9kIiwiaXNQcml2YXRlIiwicmVxdWVzdCIsInJlcGx5IiwiYWRkTnVtYmVyIiwibnVtYmVyS2luZCIsIm5hbWVzIiwicmVwZWF0ZWQiLCJDb21wb25lbnRPYmplY3QiLCJiYXNlVHlwZU5hbWUiLCJzZXRDb21wb25lbnREZWZhdWx0cyIsIm1pZ3JhdGVhYmxlIiwiYWRkR2V0TWV0aG9kIiwiYWRkTGlzdE1ldGhvZCIsImFkZE9iamVjdCIsIm11dGF0aW9uIiwiY29uc3RyYWludFByb3AiLCJnZXRFbnRyeSIsIkVudGl0eU9iamVjdCIsInNldEVudGl0eURlZmF1bHRzIiwiYWRkQWN0aW9uIiwicHJvcCIsIkVudGl0eUV2ZW50T2JqZWN0Iiwic2V0RW50aXR5RXZlbnREZWZhdWx0cyIsImFkZEJvb2wiLCJDb21wb25lbnRBbmRFbnRpdHlPYmplY3QiLCJjb21wb25lbnQiLCJlbnRpdHkiLCJlbnRpdHlFdmVudCIsImF1dG9DcmVhdGVFZGl0cyJd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QSxXQUFXLEVBQUVnQixJQUFJLENBQUNoQjtBQUpXLEtBQWpCLENBQWQ7QUFNQSxTQUFLMEQsV0FBTCxHQUFtQixJQUFJTixpQkFBSixDQUFzQjtBQUN2Q3RELE1BQUFBLFFBQVEsRUFBRWtCLElBQUksQ0FBQ2xCLFFBRHdCO0FBRXZDQyxNQUFBQSxlQUFlLEVBQUVpQixJQUFJLENBQUNqQixlQUZpQjtBQUd2Q0UsTUFBQUEsVUFBVSxFQUFFZSxJQUFJLENBQUNmLFVBSHNCO0FBSXZDRCxNQUFBQSxXQUFXLEVBQUVnQixJQUFJLENBQUNoQjtBQUpxQixLQUF0QixDQUFuQjtBQU1EOzs7O3dCQUV3RDtBQUN2RCxVQUFNbUQsSUFBSSxHQUFHLEtBQUtNLE1BQUwsQ0FBWXZDLE1BQVosQ0FBbUI2QixRQUFuQixDQUE0QixZQUE1QixDQUFiO0FBQ0FJLE1BQUFBLElBQUksQ0FBQ3RDLFVBQUwsQ0FBZ0I4QyxlQUFoQixHQUFrQyxJQUFsQztBQUNBLGFBQU9SLElBQUksQ0FBQ3RDLFVBQVo7QUFDRDs7O3dCQUU0RDtBQUMzRCxVQUFNc0MsSUFBSSxHQUFHLEtBQUtLLFNBQUwsQ0FBZXRDLE1BQWYsQ0FBc0I2QixRQUF0QixDQUErQixhQUEvQixDQUFiO0FBQ0EsYUFBT0ksSUFBSSxDQUFDdEMsVUFBWjtBQUNEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcExpbmsgfSBmcm9tIFwiLi9wcm9wL2xpbmtcIjtcbmltcG9ydCB7IFByb3BOdW1iZXIgfSBmcm9tIFwiLi9wcm9wL251bWJlclwiO1xuaW1wb3J0IHsgUHJvcE9iamVjdCwgUHJvcE1ldGhvZCwgUHJvcEFjdGlvbiB9IGZyb20gXCIuL2F0dHJMaXN0XCI7XG5pbXBvcnQgeyBjYW1lbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCB7IEFzc29jaWF0aW9uTGlzdCB9IGZyb20gXCIuL3N5c3RlbU9iamVjdC9hc3NvY2lhdGlvbnNcIjtcbmltcG9ydCB7IFNpR3JhcGhxbCB9IGZyb20gXCIuL3N5c3RlbU9iamVjdC9ncmFwaHFsXCI7XG5cbmV4cG9ydCB0eXBlIE9iamVjdFR5cGVzID1cbiAgfCBCYXNlT2JqZWN0XG4gIHwgU3lzdGVtT2JqZWN0XG4gIHwgQ29tcG9uZW50T2JqZWN0XG4gIHwgRW50aXR5T2JqZWN0XG4gIHwgRW50aXR5RXZlbnRPYmplY3Q7XG5cbmV4cG9ydCBpbnRlcmZhY2UgQmFzZU9iamVjdENvbnN0cnVjdG9yIHtcbiAgdHlwZU5hbWU6IEJhc2VPYmplY3RbXCJ0eXBlTmFtZVwiXTtcbiAgZGlzcGxheVR5cGVOYW1lOiBCYXNlT2JqZWN0W1wiZGlzcGxheVR5cGVOYW1lXCJdO1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzaVBhdGhOYW1lPzogc3RyaW5nO1xuICBvcHRpb25zPyhjOiBCYXNlT2JqZWN0KTogdm9pZDtcbn1cblxuZXhwb3J0IGludGVyZmFjZSBBZGRNZXRob2RDb25zdHJ1Y3RvciB7XG4gIGlzUHJpdmF0ZT86IFByb3BNZXRob2RbXCJpc1ByaXZhdGVcIl07XG59XG5cbmV4cG9ydCBjbGFzcyBCYXNlT2JqZWN0IHtcbiAgdHlwZU5hbWU6IHN0cmluZztcbiAgZGlzcGxheVR5cGVOYW1lOiBzdHJpbmc7XG4gIHNpUGF0aE5hbWU6IHN0cmluZztcbiAgc2VydmljZU5hbWU6IHN0cmluZztcblxuICByb290UHJvcDogUHJvcE9iamVjdDtcbiAgbWV0aG9kc1Byb3A6IFByb3BPYmplY3Q7XG4gIGFzc29jaWF0aW9uczogQXNzb2NpYXRpb25MaXN0O1xuXG4gIHByaXZhdGUgaW50ZXJuYWxHcmFwaHFsOiB1bmRlZmluZWQgfCBTaUdyYXBocWw7XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIHR5cGVOYW1lLFxuICAgIGRpc3BsYXlUeXBlTmFtZSxcbiAgICBzZXJ2aWNlTmFtZSxcbiAgICBzaVBhdGhOYW1lID0gXCJcIixcbiAgfTogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgdGhpcy50eXBlTmFtZSA9IGNhbWVsQ2FzZSh0eXBlTmFtZSk7XG4gICAgdGhpcy5kaXNwbGF5VHlwZU5hbWUgPSBkaXNwbGF5VHlwZU5hbWU7XG4gICAgdGhpcy5zaVBhdGhOYW1lID0gc2lQYXRoTmFtZTtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWUgfHwgdHlwZU5hbWU7XG4gICAgdGhpcy5yb290UHJvcCA9IG5ldyBQcm9wT2JqZWN0KHtcbiAgICAgIG5hbWU6IHR5cGVOYW1lLFxuICAgICAgbGFiZWw6IGRpc3BsYXlUeXBlTmFtZSxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0eXBlTmFtZSxcbiAgICAgIHBhcmVudE5hbWU6IFwiXCIsXG4gICAgfSk7XG4gICAgdGhpcy5tZXRob2RzUHJvcCA9IG5ldyBQcm9wT2JqZWN0KHtcbiAgICAgIG5hbWU6IGAke3R5cGVOYW1lfWAsXG4gICAgICBsYWJlbDogYCR7ZGlzcGxheVR5cGVOYW1lfSBNZXRob2RzYCxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0eXBlTmFtZSxcbiAgICAgIHBhcmVudE5hbWU6IFwiXCIsXG4gICAgfSk7XG4gICAgdGhpcy5hc3NvY2lhdGlvbnMgPSBuZXcgQXNzb2NpYXRpb25MaXN0KCk7XG4gICAgdGhpcy5pbnRlcm5hbEdyYXBocWwgPSB1bmRlZmluZWQ7XG4gIH1cblxuICBnZXQgZmllbGRzKCk6IEJhc2VPYmplY3RbXCJyb290UHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIHJldHVybiB0aGlzLnJvb3RQcm9wLnByb3BlcnRpZXM7XG4gIH1cblxuICBnZXQgbWV0aG9kcygpOiBCYXNlT2JqZWN0W1wibWV0aG9kc1Byb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICByZXR1cm4gdGhpcy5tZXRob2RzUHJvcC5wcm9wZXJ0aWVzO1xuICB9XG5cbiAgZ2V0IGdyYXBocWwoKTogU2lHcmFwaHFsIHtcbiAgICBpZiAodGhpcy5pbnRlcm5hbEdyYXBocWwgPT0gdW5kZWZpbmVkKSB7XG4gICAgICB0aGlzLmludGVybmFsR3JhcGhxbCA9IG5ldyBTaUdyYXBocWwodGhpcyk7XG4gICAgfVxuICAgIHJldHVybiB0aGlzLmludGVybmFsR3JhcGhxbDtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJiYXNlT2JqZWN0XCI7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFN5c3RlbU9iamVjdCBleHRlbmRzIEJhc2VPYmplY3Qge1xuICBuYXR1cmFsS2V5ID0gXCJuYW1lXCI7XG4gIG1pZ3JhdGVhYmxlID0gZmFsc2U7XG5cbiAgY29uc3RydWN0b3IoYXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgc3VwZXIoYXJncyk7XG4gICAgdGhpcy5zZXRTeXN0ZW1PYmplY3REZWZhdWx0cygpO1xuICB9XG5cbiAgc2V0U3lzdGVtT2JqZWN0RGVmYXVsdHMoKTogdm9pZCB7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImlkXCIsXG4gICAgICBsYWJlbDogYCR7dGhpcy5kaXNwbGF5VHlwZU5hbWV9IElEYCxcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcIm5hbWVcIixcbiAgICAgIGxhYmVsOiBgJHt0aGlzLmRpc3BsYXlUeXBlTmFtZX0gTmFtZWAsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJkaXNwbGF5TmFtZVwiLFxuICAgICAgbGFiZWw6IGAke3RoaXMuZGlzcGxheVR5cGVOYW1lfSBEaXNwbGF5IE5hbWVgLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwic2lTdG9yYWJsZVwiLFxuICAgICAgbGFiZWw6IFwiU0kgU3RvcmFibGVcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBcImRhdGFTdG9yYWJsZVwiLFxuICAgICAgICB9O1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwic3lzdGVtT2JqZWN0XCI7XG4gIH1cblxuICBhZGRHZXRNZXRob2QoYXJnczogQWRkTWV0aG9kQ29uc3RydWN0b3IgPSB7fSk6IHZvaWQge1xuICAgIC8vIGVzbGludC1kaXNhYmxlLW5leHQtbGluZVxuICAgIGNvbnN0IHN5c3RlbU9iamVjdCA9IHRoaXM7XG5cbiAgICBzeXN0ZW1PYmplY3QubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJnZXRcIixcbiAgICAgIGxhYmVsOiBgR2V0IGEgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfWAsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5pc1ByaXZhdGUgPSBhcmdzLmlzUHJpdmF0ZSB8fCBmYWxzZTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJpZFwiLFxuICAgICAgICAgIGxhYmVsOiBgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfSBJRGAsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiaXRlbVwiLFxuICAgICAgICAgIGxhYmVsOiBgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfSBJdGVtYCxcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IHN5c3RlbU9iamVjdC50eXBlTmFtZSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAgYWRkTGlzdE1ldGhvZChhcmdzOiBBZGRNZXRob2RDb25zdHJ1Y3RvciA9IHt9KTogdm9pZCB7XG4gICAgLy8gZXNsaW50LWRpc2FibGUtbmV4dC1saW5lXG4gICAgY29uc3Qgc3lzdGVtT2JqZWN0ID0gdGhpcztcbiAgICBzeXN0ZW1PYmplY3QubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJsaXN0XCIsXG4gICAgICBsYWJlbDogYExpc3QgJHtzeXN0ZW1PYmplY3QuZGlzcGxheVR5cGVOYW1lfWAsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmlzUHJpdmF0ZSA9IGFyZ3MuaXNQcml2YXRlIHx8IGZhbHNlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInF1ZXJ5XCIsXG4gICAgICAgICAgbGFiZWw6IFwiUXVlcnlcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IFwiZGF0YVF1ZXJ5XCIsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGROdW1iZXIoe1xuICAgICAgICAgIG5hbWU6IFwicGFnZVNpemVcIixcbiAgICAgICAgICBsYWJlbDogXCJQYWdlIFNpemVcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BOdW1iZXIpIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubnVtYmVyS2luZCA9IFwidWludDMyXCI7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwib3JkZXJCeVwiLFxuICAgICAgICAgIGxhYmVsOiBcIk9yZGVyIEJ5XCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwib3JkZXJCeURpcmVjdGlvblwiLFxuICAgICAgICAgIGxhYmVsOiBcIk9yZGVyIEJ5IERpcmVjdGlvblwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogXCJkYXRhUGFnZVRva2VuXCIsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJvcmRlckJ5RGlyZWN0aW9uXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJwYWdlVG9rZW5cIixcbiAgICAgICAgICBsYWJlbDogXCJQYWdlIFRva2VuXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwic2NvcGVCeVRlbmFudElkXCIsXG4gICAgICAgICAgbGFiZWw6IFwiU2NvcGUgQnkgVGVuYW50IElEXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcIml0ZW1zXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSXRlbXNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVwZWF0ZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBzeXN0ZW1PYmplY3QudHlwZU5hbWUsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTnVtYmVyKHtcbiAgICAgICAgICBuYW1lOiBcInRvdGFsQ291bnRcIixcbiAgICAgICAgICBsYWJlbDogXCJUb3RhbCBDb3VudFwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcE51bWJlcikge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5udW1iZXJLaW5kID0gXCJ1aW50MzJcIjtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwibmV4dFBhZ2VUb2tlblwiLFxuICAgICAgICAgIGxhYmVsOiBcIk5leHQgUGFnZSBUb2tlblwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgQ29tcG9uZW50T2JqZWN0IGV4dGVuZHMgU3lzdGVtT2JqZWN0IHtcbiAgYmFzZVR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3IoYXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgY29uc3QgdHlwZU5hbWUgPSBgJHthcmdzLnR5cGVOYW1lfUNvbXBvbmVudGA7XG4gICAgY29uc3QgZGlzcGxheVR5cGVOYW1lID0gYCR7YXJncy5kaXNwbGF5VHlwZU5hbWV9IENvbXBvbmVudGA7XG4gICAgc3VwZXIoe1xuICAgICAgdHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmJhc2VUeXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5zZXRDb21wb25lbnREZWZhdWx0cygpO1xuICB9XG5cbiAgc2V0Q29tcG9uZW50RGVmYXVsdHMoKTogdm9pZCB7XG4gICAgY29uc3QgYmFzZVR5cGVOYW1lID0gdGhpcy5iYXNlVHlwZU5hbWU7XG5cbiAgICB0aGlzLm1pZ3JhdGVhYmxlID0gdHJ1ZTtcblxuICAgIHRoaXMuYWRkR2V0TWV0aG9kKCk7XG4gICAgdGhpcy5hZGRMaXN0TWV0aG9kKCk7XG5cbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgIGxhYmVsOiBcIkNvbXBvbmVudCBEZXNjcmlwdGlvblwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZE9iamVjdCh7XG4gICAgICBuYW1lOiBcImNvbnN0cmFpbnRzXCIsXG4gICAgICBsYWJlbDogXCJDb21wb25lbnQgQ29uc3RyYWludHNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE9iamVjdCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICBwLnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJjb21wb25lbnROYW1lXCIsXG4gICAgICAgICAgbGFiZWw6IFwiQ29tcG9uZW50IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiY29tcG9uZW50RGlzcGxheU5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJDb21wb25lbnQgRGlzcGxheSBOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJzaVByb3BlcnRpZXNcIixcbiAgICAgIGxhYmVsOiBcIlNJIFByb3BlcnRpZXNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogXCJjb21wb25lbnRTaVByb3BlcnRpZXNcIixcbiAgICAgICAgfTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImNyZWF0ZVwiLFxuICAgICAgbGFiZWw6IFwiQ3JlYXRlIGEgQ29tcG9uZW50XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BNZXRob2QpIHtcbiAgICAgICAgcC5tdXRhdGlvbiA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5pc1ByaXZhdGUgPSB0cnVlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcIm5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJJbnRlZ3JhdGlvbiBOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJkaXNwbGF5TmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkludGVncmF0aW9uIERpc3BsYXkgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgICAgICBsYWJlbDogXCJJbnRlZ3JhdGlvbiBEaXNwbGF5IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImNvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiQ29uc3RyYWludHNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwic2lQcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiU2kgUHJvcGVydGllc1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBcImNvbXBvbmVudFNpUHJvcGVydGllc1wiLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiaXRlbVwiLFxuICAgICAgICAgIGxhYmVsOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50IEl0ZW1gLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJwaWNrXCIsXG4gICAgICBsYWJlbDogXCJQaWNrIENvbXBvbmVudFwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTWV0aG9kKSB7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgICAgICBsYWJlbDogXCJDb25zdHJhaW50c1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgICAgbmFtZXM6IFtcImNvbnN0cmFpbnRzXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiaW1wbGljaXRDb25zdHJhaW50c1wiLFxuICAgICAgICAgIGxhYmVsOiBcIkltcGxpY2l0IENvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJjb21wb25lbnRcIixcbiAgICAgICAgICBsYWJlbDogXCJDaG9zZW4gQ29tcG9uZW50XCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAgZ2V0IGNvbnN0cmFpbnRzKCk6IENvbXBvbmVudE9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgY29uc3QgY29uc3RyYWludFByb3AgPSB0aGlzLmZpZWxkcy5nZXRFbnRyeShcImNvbnN0cmFpbnRzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcmV0dXJuIGNvbnN0cmFpbnRQcm9wLnByb3BlcnRpZXM7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiY29tcG9uZW50T2JqZWN0XCI7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIEVudGl0eU9iamVjdCBleHRlbmRzIFN5c3RlbU9iamVjdCB7XG4gIGJhc2VUeXBlTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIGNvbnN0IHR5cGVOYW1lID0gYCR7YXJncy50eXBlTmFtZX1FbnRpdHlgO1xuICAgIGNvbnN0IGRpc3BsYXlUeXBlTmFtZSA9IGAke2FyZ3MuZGlzcGxheVR5cGVOYW1lfSBFbnRpdHlgO1xuICAgIHN1cGVyKHtcbiAgICAgIHR5cGVOYW1lLFxuICAgICAgZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5iYXNlVHlwZU5hbWUgPSBhcmdzLnR5cGVOYW1lO1xuICAgIHRoaXMuc2V0RW50aXR5RGVmYXVsdHMoKTtcbiAgfVxuXG4gIHNldEVudGl0eURlZmF1bHRzKCk6IHZvaWQge1xuICAgIGNvbnN0IGJhc2VUeXBlTmFtZSA9IHRoaXMuYmFzZVR5cGVOYW1lO1xuXG4gICAgdGhpcy5hZGRHZXRNZXRob2QoKTtcbiAgICB0aGlzLmFkZExpc3RNZXRob2QoKTtcblxuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJkZXNjcmlwdGlvblwiLFxuICAgICAgbGFiZWw6IFwiRW50aXR5IERlc2NyaXB0aW9uXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBcImVudGl0eVNpUHJvcGVydGllc1wiLFxuICAgICAgICB9O1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkT2JqZWN0KHtcbiAgICAgIG5hbWU6IFwicHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJjb25zdHJhaW50c1wiLFxuICAgICAgbGFiZWw6IFwiQ29uc3RyYWludHNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcImltcGxpY2l0Q29uc3RyYWludHNcIixcbiAgICAgIGxhYmVsOiBcIkltcGxpY2l0IENvbnN0cmFpbnRzXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImNyZWF0ZVwiLFxuICAgICAgbGFiZWw6IFwiQ3JlYXRlIEVudGl0eVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTWV0aG9kKSB7XG4gICAgICAgIHAubXV0YXRpb24gPSB0cnVlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcIm5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJkaXNwbGF5TmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkRpc3BsYXkgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgICAgICBsYWJlbDogXCJEZXNjcmlwdGlvblwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgICAgICBsYWJlbDogXCJDb25zdHJhaW50c1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgICAgbmFtZXM6IFtcImNvbnN0cmFpbnRzXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJwcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiUHJvcGVydGllc1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlgLFxuICAgICAgICAgICAgICBuYW1lczogW1wicHJvcGVydGllc1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwic2lQcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiU2kgUHJvcGVydGllc1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBcImVudGl0eVNpUHJvcGVydGllc1wiLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiaXRlbVwiLFxuICAgICAgICAgIGxhYmVsOiBcIiR7YmFzZVR5cGVOYW1lfUVudGl0eSBJdGVtXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiZW50aXR5RXZlbnRcIixcbiAgICAgICAgICBsYWJlbDogXCJFbnRpdHkgRXZlbnRcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eUV2ZW50YCxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5tZXRob2RzLmFkZEFjdGlvbih7XG4gICAgICBuYW1lOiBcInN5bmNcIixcbiAgICAgIGxhYmVsOiBcIlN5bmMgU3RhdGVcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcEFjdGlvbikge1xuICAgICAgICBwLm11dGF0aW9uID0gdHJ1ZTtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxuXG4gIGdldCBwcm9wZXJ0aWVzKCk6IEVudGl0eU9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgY29uc3QgcHJvcCA9IHRoaXMuZmllbGRzLmdldEVudHJ5KFwicHJvcGVydGllc1wiKSBhcyBQcm9wT2JqZWN0O1xuICAgIHJldHVybiBwcm9wLnByb3BlcnRpZXM7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiZW50aXR5T2JqZWN0XCI7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIEVudGl0eUV2ZW50T2JqZWN0IGV4dGVuZHMgU3lzdGVtT2JqZWN0IHtcbiAgYmFzZVR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3IoYXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgY29uc3QgdHlwZU5hbWUgPSBgJHthcmdzLnR5cGVOYW1lfUVudGl0eUV2ZW50YDtcbiAgICBjb25zdCBkaXNwbGF5VHlwZU5hbWUgPSBgJHthcmdzLmRpc3BsYXlUeXBlTmFtZX0gRW50aXR5RXZlbnRgO1xuICAgIHN1cGVyKHtcbiAgICAgIHR5cGVOYW1lLFxuICAgICAgZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5iYXNlVHlwZU5hbWUgPSBhcmdzLnR5cGVOYW1lO1xuICAgIHRoaXMuc2V0RW50aXR5RXZlbnREZWZhdWx0cygpO1xuICB9XG5cbiAgc2V0RW50aXR5RXZlbnREZWZhdWx0cygpOiB2b2lkIHtcbiAgICBjb25zdCBiYXNlVHlwZU5hbWUgPSB0aGlzLmJhc2VUeXBlTmFtZTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiYWN0aW9uTmFtZVwiLFxuICAgICAgbGFiZWw6IFwiQWN0aW9uIE5hbWVcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImNyZWF0ZVRpbWVcIixcbiAgICAgIGxhYmVsOiBcIkNyZWF0aW9uIFRpbWVcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwidXBkYXRlZFRpbWVcIixcbiAgICAgIGxhYmVsOiBcIlVwZGF0ZWQgVGltZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJmaW5hbFRpbWVcIixcbiAgICAgIGxhYmVsOiBcIkZpbmFsIFRpbWVcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRCb29sKHtcbiAgICAgIG5hbWU6IFwic3VjY2Vzc1wiLFxuICAgICAgbGFiZWw6IFwic3VjY2Vzc1wiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZEJvb2woe1xuICAgICAgbmFtZTogXCJmaW5hbGl6ZWRcIixcbiAgICAgIGxhYmVsOiBcIkZpbmFsaXplZFwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJ1c2VySWRcIixcbiAgICAgIGxhYmVsOiBcIlVzZXIgSURcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwib3V0cHV0TGluZXNcIixcbiAgICAgIGxhYmVsOiBcIk91dHB1dCBMaW5lc1wiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAucmVwZWF0ZWQgPSB0cnVlO1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJlcnJvckxpbmVzXCIsXG4gICAgICBsYWJlbDogXCJFcnJvciBMaW5lc1wiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAucmVwZWF0ZWQgPSB0cnVlO1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJlcnJvck1lc3NhZ2VcIixcbiAgICAgIGxhYmVsOiBcIkVycm9yIE1lc3NhZ2VcIixcbiAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJwcmV2aW91c0VudGl0eVwiLFxuICAgICAgbGFiZWw6IFwiUHJldmlvdXMgRW50aXR5XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJpbnB1dEVudGl0eVwiLFxuICAgICAgbGFiZWw6IFwiSW5wdXQgRW50aXR5XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlgLFxuICAgICAgICB9O1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwib3V0cHV0RW50aXR5XCIsXG4gICAgICBsYWJlbDogXCJPdXRwdXQgRW50aXR5XCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJzaVByb3BlcnRpZXNcIixcbiAgICAgIGxhYmVsOiBcIlNJIFByb3BlcnRpZXNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBcImVudGl0eUV2ZW50U2lQcm9wZXJ0aWVzXCIsXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5hZGRMaXN0TWV0aG9kKCk7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiZW50aXR5RXZlbnRPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgaW50ZXJmYWNlIENvbXBvbmVudEFuZEVudGl0eU9iamVjdENvbnN0cnVjdG9yIHtcbiAgdHlwZU5hbWU6IEJhc2VPYmplY3RbXCJ0eXBlTmFtZVwiXTtcbiAgZGlzcGxheVR5cGVOYW1lOiBCYXNlT2JqZWN0W1wiZGlzcGxheVR5cGVOYW1lXCJdO1xuICBzaVBhdGhOYW1lPzogc3RyaW5nO1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBvcHRpb25zPyhjOiBDb21wb25lbnRBbmRFbnRpdHlPYmplY3QpOiB2b2lkO1xufVxuXG5leHBvcnQgY2xhc3MgQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IHtcbiAgY29tcG9uZW50OiBDb21wb25lbnRPYmplY3Q7XG4gIGVudGl0eTogRW50aXR5T2JqZWN0O1xuICBlbnRpdHlFdmVudDogRW50aXR5RXZlbnRPYmplY3Q7XG5cbiAgY29uc3RydWN0b3IoYXJnczogQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICB0aGlzLmNvbXBvbmVudCA9IG5ldyBDb21wb25lbnRPYmplY3Qoe1xuICAgICAgdHlwZU5hbWU6IGFyZ3MudHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWU6IGFyZ3MuZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2lQYXRoTmFtZTogYXJncy5zaVBhdGhOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5lbnRpdHkgPSBuZXcgRW50aXR5T2JqZWN0KHtcbiAgICAgIHR5cGVOYW1lOiBhcmdzLnR5cGVOYW1lLFxuICAgICAgZGlzcGxheVR5cGVOYW1lOiBhcmdzLmRpc3BsYXlUeXBlTmFtZSxcbiAgICAgIHNpUGF0aE5hbWU6IGFyZ3Muc2lQYXRoTmFtZSxcbiAgICAgIHNlcnZpY2VOYW1lOiBhcmdzLnNlcnZpY2VOYW1lLFxuICAgIH0pO1xuICAgIHRoaXMuZW50aXR5RXZlbnQgPSBuZXcgRW50aXR5RXZlbnRPYmplY3Qoe1xuICAgICAgdHlwZU5hbWU6IGFyZ3MudHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWU6IGFyZ3MuZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2lQYXRoTmFtZTogYXJncy5zaVBhdGhOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gIH1cblxuICBnZXQgcHJvcGVydGllcygpOiBFbnRpdHlPYmplY3RbXCJyb290UHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIGNvbnN0IHByb3AgPSB0aGlzLmVudGl0eS5maWVsZHMuZ2V0RW50cnkoXCJwcm9wZXJ0aWVzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcHJvcC5wcm9wZXJ0aWVzLmF1dG9DcmVhdGVFZGl0cyA9IHRydWU7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGdldCBjb25zdHJhaW50cygpOiBDb21wb25lbnRPYmplY3RbXCJyb290UHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIGNvbnN0IHByb3AgPSB0aGlzLmNvbXBvbmVudC5maWVsZHMuZ2V0RW50cnkoXCJjb25zdHJhaW50c1wiKSBhcyBQcm9wT2JqZWN0O1xuICAgIHJldHVybiBwcm9wLnByb3BlcnRpZXM7XG4gIH1cbn1cbiJdfQ==