# coding: utf-8

"""
    System Initiative API

    The API Server for interacting with a System Initiative workspace

    The version of the OpenAPI document: 1.0.0
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest

from system_initiative_api_client.models.source_view_v1 import SourceViewV1

class TestSourceViewV1(unittest.TestCase):
    """SourceViewV1 unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> SourceViewV1:
        """Test SourceViewV1
            include_optional is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `SourceViewV1`
        """
        model = SourceViewV1()
        if include_optional:
            return SourceViewV1(
                component = '',
                prop_path = ''
            )
        else:
            return SourceViewV1(
                component = '',
                prop_path = '',
        )
        """

    def testSourceViewV1(self):
        """Test SourceViewV1"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()
