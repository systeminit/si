# coding: utf-8

"""
    System Initiative API

    The API Server for interacting with a System Initiative workspace

    The version of the OpenAPI document: 1.0.0
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest

from system_initiative_api_client.models.execute_management_function_v1_response import ExecuteManagementFunctionV1Response

class TestExecuteManagementFunctionV1Response(unittest.TestCase):
    """ExecuteManagementFunctionV1Response unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> ExecuteManagementFunctionV1Response:
        """Test ExecuteManagementFunctionV1Response
            include_optional is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `ExecuteManagementFunctionV1Response`
        """
        model = ExecuteManagementFunctionV1Response()
        if include_optional:
            return ExecuteManagementFunctionV1Response(
                func_run_id = '',
                management_func_job_state_id = '01H9ZQD35JPMBGHH69BT0Q79VY',
                message = 'enqueued',
                status = 'Ok'
            )
        else:
            return ExecuteManagementFunctionV1Response(
                func_run_id = '',
                management_func_job_state_id = '01H9ZQD35JPMBGHH69BT0Q79VY',
                status = 'Ok',
        )
        """

    def testExecuteManagementFunctionV1Response(self):
        """Test ExecuteManagementFunctionV1Response"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()
