# coding: utf-8

"""
    System Initiative API

    The API Server for interacting with a System Initiative workspace

    The version of the OpenAPI document: 1.0.0
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest

from system_initiative_api_client.api.root_api import RootApi


class TestRootApi(unittest.TestCase):
    """RootApi unit test stubs"""

    def setUp(self) -> None:
        self.api = RootApi()

    def tearDown(self) -> None:
        pass

    def test_system_status_route(self) -> None:
        """Test case for system_status_route

        """
        pass


if __name__ == '__main__':
    unittest.main()
