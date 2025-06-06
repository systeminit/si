# coding: utf-8

"""
    System Initiative API

    The API Server for interacting with a System Initiative workspace

    The version of the OpenAPI document: 1.0.0
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest

from system_initiative_api_client.models.connection_one_of1 import ConnectionOneOf1

class TestConnectionOneOf1(unittest.TestCase):
    """ConnectionOneOf1 unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> ConnectionOneOf1:
        """Test ConnectionOneOf1
            include_optional is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `ConnectionOneOf1`
        """
        model = ConnectionOneOf1()
        if include_optional:
            return ConnectionOneOf1(
                var_from = '',
                to = {componentId=01H9ZQD35JPMBGHH69BT0Q79VY, socketName=OutputSocketName}
            )
        else:
            return ConnectionOneOf1(
                var_from = '',
                to = {componentId=01H9ZQD35JPMBGHH69BT0Q79VY, socketName=OutputSocketName},
        )
        """

    def testConnectionOneOf1(self):
        """Test ConnectionOneOf1"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()
