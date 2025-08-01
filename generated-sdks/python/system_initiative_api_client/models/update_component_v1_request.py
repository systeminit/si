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

from pydantic import BaseModel, ConfigDict, Field, StrictStr
from typing import Any, ClassVar, Dict, List, Optional
from system_initiative_api_client.models.component_prop_key import ComponentPropKey
from system_initiative_api_client.models.connection_details import ConnectionDetails
from system_initiative_api_client.models.subscription import Subscription
from typing import Optional, Set
from typing_extensions import Self

class UpdateComponentV1Request(BaseModel):
    """
    UpdateComponentV1Request
    """ # noqa: E501
    attributes: Optional[Dict[str, Any]] = None
    connection_changes: Optional[ConnectionDetails] = Field(default=None, alias="connectionChanges")
    domain: Optional[Dict[str, Any]] = None
    name: Optional[StrictStr] = None
    resource_id: Optional[StrictStr] = Field(default=None, alias="resourceId")
    secrets: Optional[Dict[str, Any]] = None
    subscriptions: Optional[Dict[str, Subscription]] = None
    unset: Optional[List[ComponentPropKey]] = None
    __properties: ClassVar[List[str]] = ["attributes", "connectionChanges", "domain", "name", "resourceId", "secrets", "subscriptions", "unset"]

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
        """Create an instance of UpdateComponentV1Request from a JSON string"""
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
        # override the default output from pydantic by calling `to_dict()` of connection_changes
        if self.connection_changes:
            _dict['connectionChanges'] = self.connection_changes.to_dict()
        # override the default output from pydantic by calling `to_dict()` of each value in subscriptions (dict)
        _field_dict = {}
        if self.subscriptions:
            for _key_subscriptions in self.subscriptions:
                if self.subscriptions[_key_subscriptions]:
                    _field_dict[_key_subscriptions] = self.subscriptions[_key_subscriptions].to_dict()
            _dict['subscriptions'] = _field_dict
        # override the default output from pydantic by calling `to_dict()` of each item in unset (list)
        _items = []
        if self.unset:
            for _item_unset in self.unset:
                if _item_unset:
                    _items.append(_item_unset.to_dict())
            _dict['unset'] = _items
        # set to None if name (nullable) is None
        # and model_fields_set contains the field
        if self.name is None and "name" in self.model_fields_set:
            _dict['name'] = None

        # set to None if resource_id (nullable) is None
        # and model_fields_set contains the field
        if self.resource_id is None and "resource_id" in self.model_fields_set:
            _dict['resourceId'] = None

        return _dict

    @classmethod
    def from_dict(cls, obj: Optional[Dict[str, Any]]) -> Optional[Self]:
        """Create an instance of UpdateComponentV1Request from a dict"""
        if obj is None:
            return None

        if not isinstance(obj, dict):
            return cls.model_validate(obj)

        _obj = cls.model_validate({
            "attributes": obj.get("attributes"),
            "connectionChanges": ConnectionDetails.from_dict(obj["connectionChanges"]) if obj.get("connectionChanges") is not None else None,
            "domain": obj.get("domain"),
            "name": obj.get("name"),
            "resourceId": obj.get("resourceId"),
            "secrets": obj.get("secrets"),
            "subscriptions": dict(
                (_k, Subscription.from_dict(_v))
                for _k, _v in obj["subscriptions"].items()
            )
            if obj.get("subscriptions") is not None
            else None,
            "unset": [ComponentPropKey.from_dict(_item) for _item in obj["unset"]] if obj.get("unset") is not None else None
        })
        return _obj


