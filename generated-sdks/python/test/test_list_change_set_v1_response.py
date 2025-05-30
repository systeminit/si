# coding: utf-8

"""
    System Initiative API

    The API Server for interacting with a System Initiative workspace

    The version of the OpenAPI document: 1.0.0
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest

from system_initiative_api_client.models.list_change_set_v1_response import ListChangeSetV1Response

class TestListChangeSetV1Response(unittest.TestCase):
    """ListChangeSetV1Response unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> ListChangeSetV1Response:
        """Test ListChangeSetV1Response
            include_optional is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `ListChangeSetV1Response`
        """
        model = ListChangeSetV1Response()
        if include_optional:
            return ListChangeSetV1Response(
                change_sets = [{"id":"01H9ZQD35JPMBGHH69BT0Q79VY","name":"Add new feature","status":"Open","isHead": "false"},{"id":"01H9ZQE356JPMBGHH69BT0Q70UO","name":"HEAD","status":"Open", "isHead": "true"}]
            )
        else:
            return ListChangeSetV1Response(
                change_sets = [{"id":"01H9ZQD35JPMBGHH69BT0Q79VY","name":"Add new feature","status":"Open","isHead": "false"},{"id":"01H9ZQE356JPMBGHH69BT0Q70UO","name":"HEAD","status":"Open", "isHead": "true"}],
        )
        """

    def testListChangeSetV1Response(self):
        """Test ListChangeSetV1Response"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()
