# coding: utf-8

"""
    System Initiative API

    The API Server for interacting with a System Initiative workspace

    The version of the OpenAPI document: 1.0.0
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest

from system_initiative_api_client.models.merge_status_v1_response import MergeStatusV1Response

class TestMergeStatusV1Response(unittest.TestCase):
    """MergeStatusV1Response unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> MergeStatusV1Response:
        """Test MergeStatusV1Response
            include_optional is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `MergeStatusV1Response`
        """
        model = MergeStatusV1Response()
        if include_optional:
            return MergeStatusV1Response(
                actions = [
                    {id=01H9ZQD35JPMBGHH69BT0Q79VY, component={id=01H9ZQD35JPMBGHH69BT0Q79AB, name=my-ec2-instance}, state=Pending, kind=Create, name=Create EC2 Instance}
                    ],
                change_set = None
            )
        else:
            return MergeStatusV1Response(
                actions = [
                    {id=01H9ZQD35JPMBGHH69BT0Q79VY, component={id=01H9ZQD35JPMBGHH69BT0Q79AB, name=my-ec2-instance}, state=Pending, kind=Create, name=Create EC2 Instance}
                    ],
                change_set = None,
        )
        """

    def testMergeStatusV1Response(self):
        """Test MergeStatusV1Response"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()
