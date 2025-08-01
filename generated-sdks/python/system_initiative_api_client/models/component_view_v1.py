# coding: utf-8

"""
    System Initiative API

    The API Server for interacting with a System Initiative workspace

    The version of the OpenAPI document: 1.0.0
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


from __future__ import annotations
import pprint
import re  # noqa: F401
import json

from pydantic import BaseModel, ConfigDict, Field, StrictBool, StrictStr
from typing import Any, ClassVar, Dict, List
from system_initiative_api_client.models.component_prop_view_v1 import ComponentPropViewV1
from system_initiative_api_client.models.connection_view_v1 import ConnectionViewV1
from system_initiative_api_client.models.socket_view_v1 import SocketViewV1
from system_initiative_api_client.models.view_v1 import ViewV1
from typing import Optional, Set
from typing_extensions import Self

class ComponentViewV1(BaseModel):
    """
    ComponentViewV1
    """ # noqa: E501
    attributes: Dict[str, Any]
    can_be_upgraded: StrictBool = Field(alias="canBeUpgraded")
    connections: List[ConnectionViewV1]
    domain_props: List[ComponentPropViewV1] = Field(alias="domainProps")
    id: StrictStr
    name: StrictStr
    resource_id: StrictStr = Field(alias="resourceId")
    resource_props: List[ComponentPropViewV1] = Field(alias="resourceProps")
    schema_id: StrictStr = Field(alias="schemaId")
    schema_variant_id: StrictStr = Field(alias="schemaVariantId")
    sockets: List[SocketViewV1]
    to_delete: StrictBool = Field(alias="toDelete")
    views: List[ViewV1]
    __properties: ClassVar[List[str]] = ["attributes", "canBeUpgraded", "connections", "domainProps", "id", "name", "resourceId", "resourceProps", "schemaId", "schemaVariantId", "sockets", "toDelete", "views"]

    model_config = ConfigDict(
        populate_by_name=True,
        validate_assignment=True,
        protected_namespaces=(),
    )


    def to_str(self) -> str:
        """Returns the string representation of the model using alias"""
        return pprint.pformat(self.model_dump(by_alias=True))

    def to_json(self) -> str:
        """Returns the JSON representation of the model using alias"""
        # TODO: pydantic v2: use .model_dump_json(by_alias=True, exclude_unset=True) instead
        return json.dumps(self.to_dict())

    @classmethod
    def from_json(cls, json_str: str) -> Optional[Self]:
        """Create an instance of ComponentViewV1 from a JSON string"""
        return cls.from_dict(json.loads(json_str))

    def to_dict(self) -> Dict[str, Any]:
        """Return the dictionary representation of the model using alias.

        This has the following differences from calling pydantic's
        `self.model_dump(by_alias=True)`:

        * `None` is only added to the output dict for nullable fields that
          were set at model initialization. Other fields with value `None`
          are ignored.
        """
        excluded_fields: Set[str] = set([
        ])

        _dict = self.model_dump(
            by_alias=True,
            exclude=excluded_fields,
            exclude_none=True,
        )
        # override the default output from pydantic by calling `to_dict()` of each item in connections (list)
        _items = []
        if self.connections:
            for _item_connections in self.connections:
                if _item_connections:
                    _items.append(_item_connections.to_dict())
            _dict['connections'] = _items
        # override the default output from pydantic by calling `to_dict()` of each item in domain_props (list)
        _items = []
        if self.domain_props:
            for _item_domain_props in self.domain_props:
                if _item_domain_props:
                    _items.append(_item_domain_props.to_dict())
            _dict['domainProps'] = _items
        # override the default output from pydantic by calling `to_dict()` of each item in resource_props (list)
        _items = []
        if self.resource_props:
            for _item_resource_props in self.resource_props:
                if _item_resource_props:
                    _items.append(_item_resource_props.to_dict())
            _dict['resourceProps'] = _items
        # override the default output from pydantic by calling `to_dict()` of each item in sockets (list)
        _items = []
        if self.sockets:
            for _item_sockets in self.sockets:
                if _item_sockets:
                    _items.append(_item_sockets.to_dict())
            _dict['sockets'] = _items
        # override the default output from pydantic by calling `to_dict()` of each item in views (list)
        _items = []
        if self.views:
            for _item_views in self.views:
                if _item_views:
                    _items.append(_item_views.to_dict())
            _dict['views'] = _items
        return _dict

    @classmethod
    def from_dict(cls, obj: Optional[Dict[str, Any]]) -> Optional[Self]:
        """Create an instance of ComponentViewV1 from a dict"""
        if obj is None:
            return None

        if not isinstance(obj, dict):
            return cls.model_validate(obj)

        _obj = cls.model_validate({
            "attributes": obj.get("attributes"),
            "canBeUpgraded": obj.get("canBeUpgraded"),
            "connections": [ConnectionViewV1.from_dict(_item) for _item in obj["connections"]] if obj.get("connections") is not None else None,
            "domainProps": [ComponentPropViewV1.from_dict(_item) for _item in obj["domainProps"]] if obj.get("domainProps") is not None else None,
            "id": obj.get("id"),
            "name": obj.get("name"),
            "resourceId": obj.get("resourceId"),
            "resourceProps": [ComponentPropViewV1.from_dict(_item) for _item in obj["resourceProps"]] if obj.get("resourceProps") is not None else None,
            "schemaId": obj.get("schemaId"),
            "schemaVariantId": obj.get("schemaVariantId"),
            "sockets": [SocketViewV1.from_dict(_item) for _item in obj["sockets"]] if obj.get("sockets") is not None else None,
            "toDelete": obj.get("toDelete"),
            "views": [ViewV1.from_dict(_item) for _item in obj["views"]] if obj.get("views") is not None else None
        })
        return _obj


