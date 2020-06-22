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
          p.hidden = false;
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
              p.required = true;
              p.hidden = true;
            }
          });
          p.request.properties.addLink({
            name: "properties",
            label: "Properties",
            options: function options(p) {
              p.universal = true;
              p.readOnly = true;
              p.required = false;
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
            label: "".concat(baseTypeName, "Entity Item"),
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
      this.methods.addMethod({
        name: "delete",
        label: "Delete Entity",
        options: function options(p) {
          p.mutation = true;
          p.request.properties.addText({
            name: "id",
            label: "".concat(baseTypeName, "Entity ID"),
            options: function options(p) {
              p.required = true;
            }
          });
          p.request.properties.addText({
            name: "changeSetId",
            label: "Change Set ID",
            options: function options(p) {
              p.required = true;
              p.hidden = true;
            }
          });
          p.reply.properties.addLink({
            name: "item",
            label: "".concat(baseTypeName, " Item"),
            options: function options(p) {
              p.lookup = {
                typeName: "".concat(baseTypeName, "Entity")
              };
            }
          });
        }
      });
      this.methods.addMethod({
        name: "update",
        label: "Update an Entity",
        options: function options(p) {
          p.mutation = true;
          p.request.properties.addText({
            name: "id",
            label: "".concat(baseTypeName, "Entity ID"),
            options: function options(p) {
              p.required = true;
            }
          });
          p.request.properties.addText({
            name: "changeSetId",
            label: "Change Set ID",
            options: function options(p) {
              p.required = true;
              p.hidden = true;
            }
          });
          p.request.properties.addObject({
            name: "update",
            label: "".concat(baseTypeName, " Item Update"),
            options: function options(p) {
              p.properties.addLink({
                name: "name",
                label: "name",
                options: function options(p) {
                  p.required = false;
                  p.lookup = {
                    typeName: "".concat(baseTypeName, "Entity"),
                    names: ["name"]
                  };
                }
              });
              p.properties.addLink({
                name: "description",
                label: "description",
                options: function options(p) {
                  p.required = false;
                  p.lookup = {
                    typeName: "".concat(baseTypeName, "Entity"),
                    names: ["description"]
                  };
                }
              });
              p.properties.addLink({
                name: "properties",
                label: "properties",
                options: function options(p) {
                  p.required = false;
                  p.lookup = {
                    typeName: "".concat(baseTypeName, "Entity"),
                    names: ["properties"]
                  };
                }
              });
            }
          });
          p.reply.properties.addLink({
            name: "item",
            label: "".concat(baseTypeName, " Item"),
            options: function options(p) {
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9zeXN0ZW1Db21wb25lbnQudHMiXSwibmFtZXMiOlsiQmFzZU9iamVjdCIsInR5cGVOYW1lIiwiZGlzcGxheVR5cGVOYW1lIiwic2VydmljZU5hbWUiLCJzaVBhdGhOYW1lIiwicm9vdFByb3AiLCJQcm9wT2JqZWN0IiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJwYXJlbnROYW1lIiwibWV0aG9kc1Byb3AiLCJhc3NvY2lhdGlvbnMiLCJBc3NvY2lhdGlvbkxpc3QiLCJpbnRlcm5hbEdyYXBocWwiLCJ1bmRlZmluZWQiLCJtdmNjIiwicHJvcGVydGllcyIsIlNpR3JhcGhxbCIsIlN5c3RlbU9iamVjdCIsImFyZ3MiLCJzZXRTeXN0ZW1PYmplY3REZWZhdWx0cyIsImZpZWxkcyIsImFkZFRleHQiLCJvcHRpb25zIiwicCIsInVuaXZlcnNhbCIsInJlYWRPbmx5IiwicmVxdWlyZWQiLCJlbmRzV2l0aCIsImFkZExpbmsiLCJoaWRkZW4iLCJsb29rdXAiLCJzeXN0ZW1PYmplY3QiLCJtZXRob2RzIiwiYWRkTWV0aG9kIiwiaXNQcml2YXRlIiwicmVxdWVzdCIsInJlcGx5IiwiYWRkTnVtYmVyIiwibnVtYmVyS2luZCIsIm5hbWVzIiwicmVwZWF0ZWQiLCJDb21wb25lbnRPYmplY3QiLCJiYXNlVHlwZU5hbWUiLCJzZXRDb21wb25lbnREZWZhdWx0cyIsIm1pZ3JhdGVhYmxlIiwiYWRkR2V0TWV0aG9kIiwiYWRkTGlzdE1ldGhvZCIsImFkZE9iamVjdCIsIm11dGF0aW9uIiwiY29uc3RyYWludFByb3AiLCJnZXRFbnRyeSIsIkVudGl0eU9iamVjdCIsImludGVncmF0aW9uU2VydmljZXMiLCJzZXRFbnRpdHlEZWZhdWx0cyIsImFkZEFjdGlvbiIsInByb3AiLCJFbnRpdHlFdmVudE9iamVjdCIsInNldEVudGl0eUV2ZW50RGVmYXVsdHMiLCJhZGRCb29sIiwiQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IiwiY29tcG9uZW50Iiwi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Q0EsYUFBT0ssSUFBSSxDQUFDeEMsVUFBWjtBQUNEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcExpbmsgfSBmcm9tIFwiLi9wcm9wL2xpbmtcIjtcbmltcG9ydCB7IFByb3BOdW1iZXIgfSBmcm9tIFwiLi9wcm9wL251bWJlclwiO1xuaW1wb3J0IHtcbiAgUHJvcE9iamVjdCxcbiAgUHJvcE1ldGhvZCxcbiAgUHJvcEFjdGlvbixcbiAgSW50ZWdyYXRpb25TZXJ2aWNlLFxufSBmcm9tIFwiLi9hdHRyTGlzdFwiO1xuaW1wb3J0IHsgY2FtZWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5pbXBvcnQgeyBBc3NvY2lhdGlvbkxpc3QgfSBmcm9tIFwiLi9zeXN0ZW1PYmplY3QvYXNzb2NpYXRpb25zXCI7XG5pbXBvcnQgeyBTaUdyYXBocWwgfSBmcm9tIFwiLi9zeXN0ZW1PYmplY3QvZ3JhcGhxbFwiO1xuXG5leHBvcnQgdHlwZSBPYmplY3RUeXBlcyA9XG4gIHwgQmFzZU9iamVjdFxuICB8IFN5c3RlbU9iamVjdFxuICB8IENvbXBvbmVudE9iamVjdFxuICB8IEVudGl0eU9iamVjdFxuICB8IEVudGl0eUV2ZW50T2JqZWN0O1xuXG5leHBvcnQgaW50ZXJmYWNlIEJhc2VPYmplY3RDb25zdHJ1Y3RvciB7XG4gIHR5cGVOYW1lOiBCYXNlT2JqZWN0W1widHlwZU5hbWVcIl07XG4gIGRpc3BsYXlUeXBlTmFtZTogQmFzZU9iamVjdFtcImRpc3BsYXlUeXBlTmFtZVwiXTtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcbiAgc2lQYXRoTmFtZT86IHN0cmluZztcbiAgb3B0aW9ucz8oYzogQmFzZU9iamVjdCk6IHZvaWQ7XG59XG5cbmV4cG9ydCBpbnRlcmZhY2UgQWRkTWV0aG9kQ29uc3RydWN0b3Ige1xuICBpc1ByaXZhdGU/OiBQcm9wTWV0aG9kW1wiaXNQcml2YXRlXCJdO1xufVxuXG5leHBvcnQgY2xhc3MgQmFzZU9iamVjdCB7XG4gIHR5cGVOYW1lOiBzdHJpbmc7XG4gIGRpc3BsYXlUeXBlTmFtZTogc3RyaW5nO1xuICBzaVBhdGhOYW1lOiBzdHJpbmc7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIG12Y2M6IGJvb2xlYW47XG5cbiAgcm9vdFByb3A6IFByb3BPYmplY3Q7XG4gIG1ldGhvZHNQcm9wOiBQcm9wT2JqZWN0O1xuICBhc3NvY2lhdGlvbnM6IEFzc29jaWF0aW9uTGlzdDtcblxuICBwcml2YXRlIGludGVybmFsR3JhcGhxbDogdW5kZWZpbmVkIHwgU2lHcmFwaHFsO1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICB0eXBlTmFtZSxcbiAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgc2VydmljZU5hbWUsXG4gICAgc2lQYXRoTmFtZSA9IFwiXCIsXG4gIH06IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIHRoaXMudHlwZU5hbWUgPSBjYW1lbENhc2UodHlwZU5hbWUpO1xuICAgIHRoaXMuZGlzcGxheVR5cGVOYW1lID0gZGlzcGxheVR5cGVOYW1lO1xuICAgIHRoaXMuc2lQYXRoTmFtZSA9IHNpUGF0aE5hbWU7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lIHx8IHR5cGVOYW1lO1xuICAgIHRoaXMucm9vdFByb3AgPSBuZXcgUHJvcE9iamVjdCh7XG4gICAgICBuYW1lOiB0eXBlTmFtZSxcbiAgICAgIGxhYmVsOiBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBjb21wb25lbnRUeXBlTmFtZTogdHlwZU5hbWUsXG4gICAgICBwYXJlbnROYW1lOiBcIlwiLFxuICAgIH0pO1xuICAgIHRoaXMubWV0aG9kc1Byb3AgPSBuZXcgUHJvcE9iamVjdCh7XG4gICAgICBuYW1lOiBgJHt0eXBlTmFtZX1gLFxuICAgICAgbGFiZWw6IGAke2Rpc3BsYXlUeXBlTmFtZX0gTWV0aG9kc2AsXG4gICAgICBjb21wb25lbnRUeXBlTmFtZTogdHlwZU5hbWUsXG4gICAgICBwYXJlbnROYW1lOiBcIlwiLFxuICAgIH0pO1xuICAgIHRoaXMuYXNzb2NpYXRpb25zID0gbmV3IEFzc29jaWF0aW9uTGlzdCgpO1xuICAgIHRoaXMuaW50ZXJuYWxHcmFwaHFsID0gdW5kZWZpbmVkO1xuICAgIHRoaXMubXZjYyA9IGZhbHNlO1xuICB9XG5cbiAgZ2V0IGZpZWxkcygpOiBCYXNlT2JqZWN0W1wicm9vdFByb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICByZXR1cm4gdGhpcy5yb290UHJvcC5wcm9wZXJ0aWVzO1xuICB9XG5cbiAgZ2V0IG1ldGhvZHMoKTogQmFzZU9iamVjdFtcIm1ldGhvZHNQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgcmV0dXJuIHRoaXMubWV0aG9kc1Byb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGdldCBncmFwaHFsKCk6IFNpR3JhcGhxbCB7XG4gICAgaWYgKHRoaXMuaW50ZXJuYWxHcmFwaHFsID09IHVuZGVmaW5lZCkge1xuICAgICAgdGhpcy5pbnRlcm5hbEdyYXBocWwgPSBuZXcgU2lHcmFwaHFsKHRoaXMpO1xuICAgIH1cbiAgICByZXR1cm4gdGhpcy5pbnRlcm5hbEdyYXBocWw7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiYmFzZU9iamVjdFwiO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBTeXN0ZW1PYmplY3QgZXh0ZW5kcyBCYXNlT2JqZWN0IHtcbiAgbmF0dXJhbEtleSA9IFwibmFtZVwiO1xuICBtaWdyYXRlYWJsZSA9IGZhbHNlO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIHN1cGVyKGFyZ3MpO1xuICAgIHRoaXMuc2V0U3lzdGVtT2JqZWN0RGVmYXVsdHMoKTtcbiAgfVxuXG4gIHNldFN5c3RlbU9iamVjdERlZmF1bHRzKCk6IHZvaWQge1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJpZFwiLFxuICAgICAgbGFiZWw6IGAke3RoaXMuZGlzcGxheVR5cGVOYW1lfSBJRGAsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIGlmICghdGhpcy50eXBlTmFtZS5lbmRzV2l0aChcIkVudGl0eUV2ZW50XCIpKSB7XG4gICAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgICAgbmFtZTogXCJuYW1lXCIsXG4gICAgICAgIGxhYmVsOiBgJHt0aGlzLmRpc3BsYXlUeXBlTmFtZX0gTmFtZWAsXG4gICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgfSxcbiAgICAgIH0pO1xuICAgICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICAgIG5hbWU6IFwiZGlzcGxheU5hbWVcIixcbiAgICAgICAgbGFiZWw6IGAke3RoaXMuZGlzcGxheVR5cGVOYW1lfSBEaXNwbGF5IE5hbWVgLFxuICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIH0sXG4gICAgICB9KTtcbiAgICB9XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpU3RvcmFibGVcIixcbiAgICAgIGxhYmVsOiBcIlNJIFN0b3JhYmxlXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5oaWRkZW4gPSBmYWxzZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IFwiZGF0YVN0b3JhYmxlXCIsXG4gICAgICAgIH07XG4gICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJzeXN0ZW1PYmplY3RcIjtcbiAgfVxuXG4gIGFkZEdldE1ldGhvZChhcmdzOiBBZGRNZXRob2RDb25zdHJ1Y3RvciA9IHt9KTogdm9pZCB7XG4gICAgLy8gZXNsaW50LWRpc2FibGUtbmV4dC1saW5lXG4gICAgY29uc3Qgc3lzdGVtT2JqZWN0ID0gdGhpcztcblxuICAgIHN5c3RlbU9iamVjdC5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImdldFwiLFxuICAgICAgbGFiZWw6IGBHZXQgYSAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9YCxcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLmlzUHJpdmF0ZSA9IGFyZ3MuaXNQcml2YXRlIHx8IGZhbHNlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImlkXCIsXG4gICAgICAgICAgbGFiZWw6IGAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9IElEYCxcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpdGVtXCIsXG4gICAgICAgICAgbGFiZWw6IGAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9IEl0ZW1gLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogc3lzdGVtT2JqZWN0LnR5cGVOYW1lLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cblxuICBhZGRMaXN0TWV0aG9kKGFyZ3M6IEFkZE1ldGhvZENvbnN0cnVjdG9yID0ge30pOiB2b2lkIHtcbiAgICAvLyBlc2xpbnQtZGlzYWJsZS1uZXh0LWxpbmVcbiAgICBjb25zdCBzeXN0ZW1PYmplY3QgPSB0aGlzO1xuICAgIHN5c3RlbU9iamVjdC5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImxpc3RcIixcbiAgICAgIGxhYmVsOiBgTGlzdCAke3N5c3RlbU9iamVjdC5kaXNwbGF5VHlwZU5hbWV9YCxcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaXNQcml2YXRlID0gYXJncy5pc1ByaXZhdGUgfHwgZmFsc2U7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwicXVlcnlcIixcbiAgICAgICAgICBsYWJlbDogXCJRdWVyeVwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogXCJkYXRhUXVlcnlcIixcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZE51bWJlcih7XG4gICAgICAgICAgbmFtZTogXCJwYWdlU2l6ZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIlBhZ2UgU2l6ZVwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcE51bWJlcikge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5udW1iZXJLaW5kID0gXCJ1aW50MzJcIjtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJvcmRlckJ5XCIsXG4gICAgICAgICAgbGFiZWw6IFwiT3JkZXIgQnlcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJvcmRlckJ5RGlyZWN0aW9uXCIsXG4gICAgICAgICAgbGFiZWw6IFwiT3JkZXIgQnkgRGlyZWN0aW9uXCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBcImRhdGFQYWdlVG9rZW5cIixcbiAgICAgICAgICAgICAgbmFtZXM6IFtcIm9yZGVyQnlEaXJlY3Rpb25cIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcInBhZ2VUb2tlblwiLFxuICAgICAgICAgIGxhYmVsOiBcIlBhZ2UgVG9rZW5cIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJzY29wZUJ5VGVuYW50SWRcIixcbiAgICAgICAgICBsYWJlbDogXCJTY29wZSBCeSBUZW5hbnQgSURcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiaXRlbXNcIixcbiAgICAgICAgICBsYWJlbDogXCJJdGVtc1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXBlYXRlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IHN5c3RlbU9iamVjdC50eXBlTmFtZSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGROdW1iZXIoe1xuICAgICAgICAgIG5hbWU6IFwidG90YWxDb3VudFwiLFxuICAgICAgICAgIGxhYmVsOiBcIlRvdGFsIENvdW50XCIsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTnVtYmVyKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLm51bWJlcktpbmQgPSBcInVpbnQzMlwiO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJuZXh0UGFnZVRva2VuXCIsXG4gICAgICAgICAgbGFiZWw6IFwiTmV4dCBQYWdlIFRva2VuXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBDb21wb25lbnRPYmplY3QgZXh0ZW5kcyBTeXN0ZW1PYmplY3Qge1xuICBiYXNlVHlwZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICBjb25zdCB0eXBlTmFtZSA9IGAke2FyZ3MudHlwZU5hbWV9Q29tcG9uZW50YDtcbiAgICBjb25zdCBkaXNwbGF5VHlwZU5hbWUgPSBgJHthcmdzLmRpc3BsYXlUeXBlTmFtZX0gQ29tcG9uZW50YDtcbiAgICBzdXBlcih7XG4gICAgICB0eXBlTmFtZSxcbiAgICAgIGRpc3BsYXlUeXBlTmFtZSxcbiAgICAgIHNlcnZpY2VOYW1lOiBhcmdzLnNlcnZpY2VOYW1lLFxuICAgIH0pO1xuICAgIHRoaXMuYmFzZVR5cGVOYW1lID0gYXJncy50eXBlTmFtZTtcbiAgICB0aGlzLnNldENvbXBvbmVudERlZmF1bHRzKCk7XG4gIH1cblxuICBzZXRDb21wb25lbnREZWZhdWx0cygpOiB2b2lkIHtcbiAgICBjb25zdCBiYXNlVHlwZU5hbWUgPSB0aGlzLmJhc2VUeXBlTmFtZTtcblxuICAgIHRoaXMubWlncmF0ZWFibGUgPSB0cnVlO1xuXG4gICAgdGhpcy5hZGRHZXRNZXRob2QoKTtcbiAgICB0aGlzLmFkZExpc3RNZXRob2QoKTtcblxuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJkZXNjcmlwdGlvblwiLFxuICAgICAgbGFiZWw6IFwiQ29tcG9uZW50IERlc2NyaXB0aW9uXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkT2JqZWN0KHtcbiAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgIGxhYmVsOiBcIkNvbXBvbmVudCBDb25zdHJhaW50c1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wT2JqZWN0KSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIHAucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImNvbXBvbmVudE5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJDb21wb25lbnQgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJjb21wb25lbnREaXNwbGF5TmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkNvbXBvbmVudCBEaXNwbGF5IE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBcImNvbXBvbmVudFNpUHJvcGVydGllc1wiLFxuICAgICAgICB9O1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkTWV0aG9kKHtcbiAgICAgIG5hbWU6IFwiY3JlYXRlXCIsXG4gICAgICBsYWJlbDogXCJDcmVhdGUgYSBDb21wb25lbnRcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLm11dGF0aW9uID0gdHJ1ZTtcbiAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICBwLmlzUHJpdmF0ZSA9IHRydWU7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwibmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkludGVncmF0aW9uIE5hbWVcIixcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImRpc3BsYXlOYW1lXCIsXG4gICAgICAgICAgbGFiZWw6IFwiSW50ZWdyYXRpb24gRGlzcGxheSBOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJkZXNjcmlwdGlvblwiLFxuICAgICAgICAgIGxhYmVsOiBcIkludGVncmF0aW9uIERpc3BsYXkgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgICAgICBsYWJlbDogXCJDb25zdHJhaW50c1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwic2lQcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiU2kgUHJvcGVydGllc1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBcImNvbXBvbmVudFNpUHJvcGVydGllc1wiLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiaXRlbVwiLFxuICAgICAgICAgIGxhYmVsOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50IEl0ZW1gLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJwaWNrXCIsXG4gICAgICBsYWJlbDogXCJQaWNrIENvbXBvbmVudFwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTWV0aG9kKSB7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiY29uc3RyYWludHNcIixcbiAgICAgICAgICBsYWJlbDogXCJDb25zdHJhaW50c1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImltcGxpY2l0Q29uc3RyYWludHNcIixcbiAgICAgICAgICBsYWJlbDogXCJJbXBsaWNpdCBDb25zdHJhaW50c1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICAgICAgbmFtZXM6IFtcImNvbnN0cmFpbnRzXCJdLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwiY29tcG9uZW50XCIsXG4gICAgICAgICAgbGFiZWw6IFwiQ2hvc2VuIENvbXBvbmVudFwiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxuXG4gIGdldCBjb25zdHJhaW50cygpOiBDb21wb25lbnRPYmplY3RbXCJyb290UHJvcFwiXVtcInByb3BlcnRpZXNcIl0ge1xuICAgIGNvbnN0IGNvbnN0cmFpbnRQcm9wID0gdGhpcy5maWVsZHMuZ2V0RW50cnkoXCJjb25zdHJhaW50c1wiKSBhcyBQcm9wT2JqZWN0O1xuICAgIHJldHVybiBjb25zdHJhaW50UHJvcC5wcm9wZXJ0aWVzO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImNvbXBvbmVudE9iamVjdFwiO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBFbnRpdHlPYmplY3QgZXh0ZW5kcyBTeXN0ZW1PYmplY3Qge1xuICBiYXNlVHlwZU5hbWU6IHN0cmluZztcbiAgaW50ZWdyYXRpb25TZXJ2aWNlczogSW50ZWdyYXRpb25TZXJ2aWNlW107XG5cbiAgY29uc3RydWN0b3IoYXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKSB7XG4gICAgY29uc3QgdHlwZU5hbWUgPSBgJHthcmdzLnR5cGVOYW1lfUVudGl0eWA7XG4gICAgY29uc3QgZGlzcGxheVR5cGVOYW1lID0gYCR7YXJncy5kaXNwbGF5VHlwZU5hbWV9IEVudGl0eWA7XG4gICAgc3VwZXIoe1xuICAgICAgdHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmJhc2VUeXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5pbnRlZ3JhdGlvblNlcnZpY2VzID0gW107XG4gICAgdGhpcy5zZXRFbnRpdHlEZWZhdWx0cygpO1xuICB9XG5cbiAgc2V0RW50aXR5RGVmYXVsdHMoKTogdm9pZCB7XG4gICAgY29uc3QgYmFzZVR5cGVOYW1lID0gdGhpcy5iYXNlVHlwZU5hbWU7XG5cbiAgICB0aGlzLm12Y2MgPSB0cnVlO1xuXG4gICAgdGhpcy5hZGRHZXRNZXRob2QoKTtcbiAgICB0aGlzLmFkZExpc3RNZXRob2QoKTtcblxuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJkZXNjcmlwdGlvblwiLFxuICAgICAgbGFiZWw6IFwiRW50aXR5IERlc2NyaXB0aW9uXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBcImVudGl0eVNpUHJvcGVydGllc1wiLFxuICAgICAgICB9O1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkT2JqZWN0KHtcbiAgICAgIG5hbWU6IFwicHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJjb25zdHJhaW50c1wiLFxuICAgICAgbGFiZWw6IFwiQ29uc3RyYWludHNcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1Db21wb25lbnRgLFxuICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcImltcGxpY2l0Q29uc3RyYWludHNcIixcbiAgICAgIGxhYmVsOiBcIkltcGxpY2l0IENvbnN0cmFpbnRzXCIsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9Q29tcG9uZW50YCxcbiAgICAgICAgICBuYW1lczogW1wiY29uc3RyYWludHNcIl0sXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcImNyZWF0ZVwiLFxuICAgICAgbGFiZWw6IFwiQ3JlYXRlIEVudGl0eVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTWV0aG9kKSB7XG4gICAgICAgIHAubXV0YXRpb24gPSB0cnVlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcIm5hbWVcIixcbiAgICAgICAgICBsYWJlbDogXCJOYW1lXCIsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJkaXNwbGF5TmFtZVwiLFxuICAgICAgICAgIGxhYmVsOiBcIkRpc3BsYXkgTmFtZVwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgICAgICBsYWJlbDogXCJEZXNjcmlwdGlvblwiLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwid29ya3NwYWNlSWRcIixcbiAgICAgICAgICBsYWJlbDogYFdvcmtzcGFjZSBJRGAsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJjaGFuZ2VTZXRJZFwiLFxuICAgICAgICAgIGxhYmVsOiBgQ2hhbmdlIFNldCBJRGAsXG4gICAgICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJwcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiUHJvcGVydGllc1wiLFxuICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IGZhbHNlO1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgICAgICAgbmFtZXM6IFtcInByb3BlcnRpZXNcIl0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcImNvbnN0cmFpbnRzXCIsXG4gICAgICAgICAgbGFiZWw6IFwiQ29uc3RyYWludHNcIixcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUNvbXBvbmVudGAsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJjb25zdHJhaW50c1wiXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcIml0ZW1cIixcbiAgICAgICAgICBsYWJlbDogYCR7YmFzZVR5cGVOYW1lfUVudGl0eSBJdGVtYCxcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcblxuICAgIHRoaXMubWV0aG9kcy5hZGRNZXRob2Qoe1xuICAgICAgbmFtZTogXCJkZWxldGVcIixcbiAgICAgIGxhYmVsOiBcIkRlbGV0ZSBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcE1ldGhvZCkge1xuICAgICAgICBwLm11dGF0aW9uID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1ZXN0LnByb3BlcnRpZXMuYWRkVGV4dCh7XG4gICAgICAgICAgbmFtZTogXCJpZFwiLFxuICAgICAgICAgIGxhYmVsOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5IElEYCxcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImNoYW5nZVNldElkXCIsXG4gICAgICAgICAgbGFiZWw6IGBDaGFuZ2UgU2V0IElEYCxcbiAgICAgICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgICAgIHAucmVxdWlyZWQgPSB0cnVlO1xuICAgICAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgICBwLnJlcGx5LnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgbmFtZTogXCJpdGVtXCIsXG4gICAgICAgICAgbGFiZWw6IGAke2Jhc2VUeXBlTmFtZX0gSXRlbWAsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuXG4gICAgdGhpcy5tZXRob2RzLmFkZE1ldGhvZCh7XG4gICAgICBuYW1lOiBcInVwZGF0ZVwiLFxuICAgICAgbGFiZWw6IFwiVXBkYXRlIGFuIEVudGl0eVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTWV0aG9kKSB7XG4gICAgICAgIHAubXV0YXRpb24gPSB0cnVlO1xuICAgICAgICBwLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgICAgICBuYW1lOiBcImlkXCIsXG4gICAgICAgICAgbGFiZWw6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHkgSURgLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgICAgIG5hbWU6IFwiY2hhbmdlU2V0SWRcIixcbiAgICAgICAgICBsYWJlbDogYENoYW5nZSBTZXQgSURgLFxuICAgICAgICAgIG9wdGlvbnMocCkge1xuICAgICAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZE9iamVjdCh7XG4gICAgICAgICAgbmFtZTogXCJ1cGRhdGVcIixcbiAgICAgICAgICBsYWJlbDogYCR7YmFzZVR5cGVOYW1lfSBJdGVtIFVwZGF0ZWAsXG4gICAgICAgICAgb3B0aW9ucyhwOiBQcm9wT2JqZWN0KSB7XG4gICAgICAgICAgICBwLnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgICAgIG5hbWU6IFwibmFtZVwiLFxuICAgICAgICAgICAgICBsYWJlbDogXCJuYW1lXCIsXG4gICAgICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgICAgICBwLnJlcXVpcmVkID0gZmFsc2U7XG4gICAgICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgICAgICAgICAgICBuYW1lczogW1wibmFtZVwiXSxcbiAgICAgICAgICAgICAgICB9O1xuICAgICAgICAgICAgICB9LFxuICAgICAgICAgICAgfSk7XG4gICAgICAgICAgICBwLnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgICAgIG5hbWU6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgICAgICAgICAgbGFiZWw6IFwiZGVzY3JpcHRpb25cIixcbiAgICAgICAgICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICAgICAgICAgIHAucmVxdWlyZWQgPSBmYWxzZTtcbiAgICAgICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgICAgICAgICAgIG5hbWVzOiBbXCJkZXNjcmlwdGlvblwiXSxcbiAgICAgICAgICAgICAgICB9O1xuICAgICAgICAgICAgICB9LFxuICAgICAgICAgICAgfSk7XG4gICAgICAgICAgICBwLnByb3BlcnRpZXMuYWRkTGluayh7XG4gICAgICAgICAgICAgIG5hbWU6IFwicHJvcGVydGllc1wiLFxuICAgICAgICAgICAgICBsYWJlbDogXCJwcm9wZXJ0aWVzXCIsXG4gICAgICAgICAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgICAgICAgICBwLnJlcXVpcmVkID0gZmFsc2U7XG4gICAgICAgICAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgICAgICAgICAgICBuYW1lczogW1wicHJvcGVydGllc1wiXSxcbiAgICAgICAgICAgICAgICB9O1xuICAgICAgICAgICAgICB9LFxuICAgICAgICAgICAgfSk7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICAgIHAucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcIml0ZW1cIixcbiAgICAgICAgICBsYWJlbDogYCR7YmFzZVR5cGVOYW1lfSBJdGVtYCxcbiAgICAgICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IGAke2Jhc2VUeXBlTmFtZX1FbnRpdHlgLFxuICAgICAgICAgICAgfTtcbiAgICAgICAgICB9LFxuICAgICAgICB9KTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLm1ldGhvZHMuYWRkQWN0aW9uKHtcbiAgICAgIG5hbWU6IFwic3luY1wiLFxuICAgICAgbGFiZWw6IFwiU3luYyBTdGF0ZVwiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wQWN0aW9uKSB7XG4gICAgICAgIHAubXV0YXRpb24gPSB0cnVlO1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAgZ2V0IHByb3BlcnRpZXMoKTogRW50aXR5T2JqZWN0W1wicm9vdFByb3BcIl1bXCJwcm9wZXJ0aWVzXCJdIHtcbiAgICBjb25zdCBwcm9wID0gdGhpcy5maWVsZHMuZ2V0RW50cnkoXCJwcm9wZXJ0aWVzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJlbnRpdHlPYmplY3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgRW50aXR5RXZlbnRPYmplY3QgZXh0ZW5kcyBTeXN0ZW1PYmplY3Qge1xuICBiYXNlVHlwZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpIHtcbiAgICBjb25zdCB0eXBlTmFtZSA9IGAke2FyZ3MudHlwZU5hbWV9RW50aXR5RXZlbnRgO1xuICAgIGNvbnN0IGRpc3BsYXlUeXBlTmFtZSA9IGAke2FyZ3MuZGlzcGxheVR5cGVOYW1lfSBFbnRpdHlFdmVudGA7XG4gICAgc3VwZXIoe1xuICAgICAgdHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmJhc2VUeXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5zZXRFbnRpdHlFdmVudERlZmF1bHRzKCk7XG4gIH1cblxuICBzZXRFbnRpdHlFdmVudERlZmF1bHRzKCk6IHZvaWQge1xuICAgIGNvbnN0IGJhc2VUeXBlTmFtZSA9IHRoaXMuYmFzZVR5cGVOYW1lO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJhY3Rpb25OYW1lXCIsXG4gICAgICBsYWJlbDogXCJBY3Rpb24gTmFtZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgfSxcbiAgICB9KTtcbiAgICB0aGlzLmZpZWxkcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiY3JlYXRlVGltZVwiLFxuICAgICAgbGFiZWw6IFwiQ3JlYXRpb24gVGltZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJ1cGRhdGVkVGltZVwiLFxuICAgICAgbGFiZWw6IFwiVXBkYXRlZCBUaW1lXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImZpbmFsVGltZVwiLFxuICAgICAgbGFiZWw6IFwiRmluYWwgVGltZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZEJvb2woe1xuICAgICAgbmFtZTogXCJzdWNjZXNzXCIsXG4gICAgICBsYWJlbDogXCJzdWNjZXNzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkQm9vbCh7XG4gICAgICBuYW1lOiBcImZpbmFsaXplZFwiLFxuICAgICAgbGFiZWw6IFwiRmluYWxpemVkXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcInVzZXJJZFwiLFxuICAgICAgbGFiZWw6IFwiVXNlciBJRFwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJvdXRwdXRMaW5lc1wiLFxuICAgICAgbGFiZWw6IFwiT3V0cHV0IExpbmVzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC5yZXBlYXRlZCA9IHRydWU7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImVycm9yTGluZXNcIixcbiAgICAgIGxhYmVsOiBcIkVycm9yIExpbmVzXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC5yZXBlYXRlZCA9IHRydWU7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkVGV4dCh7XG4gICAgICBuYW1lOiBcImVycm9yTWVzc2FnZVwiLFxuICAgICAgbGFiZWw6IFwiRXJyb3IgTWVzc2FnZVwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInByZXZpb3VzRW50aXR5XCIsXG4gICAgICBsYWJlbDogXCJQcmV2aW91cyBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcImlucHV0RW50aXR5XCIsXG4gICAgICBsYWJlbDogXCJJbnB1dCBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgICAgcC5oaWRkZW4gPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7YmFzZVR5cGVOYW1lfUVudGl0eWAsXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMuZmllbGRzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJvdXRwdXRFbnRpdHlcIixcbiAgICAgIGxhYmVsOiBcIk91dHB1dCBFbnRpdHlcIixcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLmhpZGRlbiA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHtiYXNlVHlwZU5hbWV9RW50aXR5YCxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5maWVsZHMuYWRkTGluayh7XG4gICAgICBuYW1lOiBcInNpUHJvcGVydGllc1wiLFxuICAgICAgbGFiZWw6IFwiU0kgUHJvcGVydGllc1wiLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAuaGlkZGVuID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IFwiZW50aXR5RXZlbnRTaVByb3BlcnRpZXNcIixcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG5cbiAgICB0aGlzLmFkZExpc3RNZXRob2QoKTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJlbnRpdHlFdmVudE9iamVjdFwiO1xuICB9XG59XG5cbmV4cG9ydCBpbnRlcmZhY2UgQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0Q29uc3RydWN0b3Ige1xuICB0eXBlTmFtZTogQmFzZU9iamVjdFtcInR5cGVOYW1lXCJdO1xuICBkaXNwbGF5VHlwZU5hbWU6IEJhc2VPYmplY3RbXCJkaXNwbGF5VHlwZU5hbWVcIl07XG4gIHNpUGF0aE5hbWU/OiBzdHJpbmc7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIG9wdGlvbnM/KGM6IENvbXBvbmVudEFuZEVudGl0eU9iamVjdCk6IHZvaWQ7XG59XG5cbmV4cG9ydCBjbGFzcyBDb21wb25lbnRBbmRFbnRpdHlPYmplY3Qge1xuICBjb21wb25lbnQ6IENvbXBvbmVudE9iamVjdDtcbiAgZW50aXR5OiBFbnRpdHlPYmplY3Q7XG4gIGVudGl0eUV2ZW50OiBFbnRpdHlFdmVudE9iamVjdDtcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBDb21wb25lbnRBbmRFbnRpdHlPYmplY3RDb25zdHJ1Y3Rvcikge1xuICAgIHRoaXMuY29tcG9uZW50ID0gbmV3IENvbXBvbmVudE9iamVjdCh7XG4gICAgICB0eXBlTmFtZTogYXJncy50eXBlTmFtZSxcbiAgICAgIGRpc3BsYXlUeXBlTmFtZTogYXJncy5kaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzaVBhdGhOYW1lOiBhcmdzLnNpUGF0aE5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLmVudGl0eSA9IG5ldyBFbnRpdHlPYmplY3Qoe1xuICAgICAgdHlwZU5hbWU6IGFyZ3MudHlwZU5hbWUsXG4gICAgICBkaXNwbGF5VHlwZU5hbWU6IGFyZ3MuZGlzcGxheVR5cGVOYW1lLFxuICAgICAgc2lQYXRoTmFtZTogYXJncy5zaVBhdGhOYW1lLFxuICAgICAgc2VydmljZU5hbWU6IGFyZ3Muc2VydmljZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5lbnRpdHlFdmVudCA9IG5ldyBFbnRpdHlFdmVudE9iamVjdCh7XG4gICAgICB0eXBlTmFtZTogYXJncy50eXBlTmFtZSxcbiAgICAgIGRpc3BsYXlUeXBlTmFtZTogYXJncy5kaXNwbGF5VHlwZU5hbWUsXG4gICAgICBzaVBhdGhOYW1lOiBhcmdzLnNpUGF0aE5hbWUsXG4gICAgICBzZXJ2aWNlTmFtZTogYXJncy5zZXJ2aWNlTmFtZSxcbiAgICB9KTtcbiAgfVxuXG4gIGdldCBwcm9wZXJ0aWVzKCk6IEVudGl0eU9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgY29uc3QgcHJvcCA9IHRoaXMuZW50aXR5LmZpZWxkcy5nZXRFbnRyeShcInByb3BlcnRpZXNcIikgYXMgUHJvcE9iamVjdDtcbiAgICBwcm9wLnByb3BlcnRpZXMuYXV0b0NyZWF0ZUVkaXRzID0gdHJ1ZTtcbiAgICByZXR1cm4gcHJvcC5wcm9wZXJ0aWVzO1xuICB9XG5cbiAgZ2V0IGNvbnN0cmFpbnRzKCk6IENvbXBvbmVudE9iamVjdFtcInJvb3RQcm9wXCJdW1wicHJvcGVydGllc1wiXSB7XG4gICAgY29uc3QgcHJvcCA9IHRoaXMuY29tcG9uZW50LmZpZWxkcy5nZXRFbnRyeShcImNvbnN0cmFpbnRzXCIpIGFzIFByb3BPYmplY3Q7XG4gICAgcmV0dXJuIHByb3AucHJvcGVydGllcztcbiAgfVxufVxuIl19