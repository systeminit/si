# Read through all files in current directory
import csv
import os
import re
from typing import Literal, Optional

def read_workspace_list(file: str):
    with open(file, 'r') as f:
        return f.read().splitlines()

def is_schema(expected: str | list[str], actual: str):
    if isinstance(expected, str):
        return actual.startswith(expected) and (len(actual) == len(expected) or actual[len(expected)] == ' ')
    else:
        return any(is_schema(e, actual) for e in expected)

ExpectedSocket = tuple[str | list[str], str | list[str]]
ExpectedSockets = ExpectedSocket | list[ExpectedSocket]

def is_socket(prefix: Literal['Source', 'Dest'], row: dict[str, str], expected: ExpectedSockets):
    expected = [expected] if isinstance(expected, tuple) else expected
    for expected_schema, expected_socket in expected:
        actual_schema = row[f"{prefix}Schema"]
        if is_schema(expected_schema, actual_schema):
            expected_socket = [expected_socket] if isinstance(expected_socket, str) else expected_socket
            actual_socket = row[f"{prefix}Socket"]
            if actual_socket in expected_socket:
                return True
    return False

def source_is(row: dict[str, str], expected: ExpectedSockets):
    return is_socket('Source', row, expected)

def dest_is(row: dict[str, str], expected: ExpectedSockets):
    return is_socket('Dest', row, expected)

def assert_source(row: dict[str, str], expected: ExpectedSockets):
    assert source_is(row, expected), f"{row['SourceSocket']} on {row['SourceSchema']} is not {expected}\n{row}"

def assert_dest(row: dict[str, str], expected: ExpectedSockets):
    assert dest_is(row, expected), f"{row['DestSocket']} on {row['DestSchema']} is not {expected}\n{row}"

def assert_error(row: dict[str, str], expected_errors: Optional[str] | list[Optional[str]], *, source_func: Optional[str | list[str]] = None, dest_func: Optional[str | list[str]] = None):
    expected_errors = [expected_errors] if isinstance(expected_errors, str) or expected_errors == None else expected_errors
    assert row['Error'] in expected_errors, f"Unexpected error {row['Error']}"
    if row.get('SourceFunction') is not None and source_func is not None:
        source_func = [source_func] if isinstance(source_func, str) else source_func
        assert row['SourceFunction'] in source_func, f"Unexpected source function {row['SourceFunction']}"
    if row.get('DestFunction') is not None and dest_func is not None:
        dest_func = [dest_func] if isinstance(dest_func, str) else dest_func
        assert row['DestFunction'] in dest_func, f"Unexpected destination function {row['DestFunction']}"

def category_of(row):
    # Likely a graph error! Look into it
    if row['Error'] == 'Source socket prototype has no arguments':
        return None

    if row['Error'] == 'Multiple connections to the same prop':
        # NOTE we should look at each of these!
        return f"FIXED-ENG-3151: remove connections with graph errors"

    # ENG-3146: unmarked si:identity functions
    if dest_is(row, [
        ('ECS Load Balancer Configuration', 'Container Name'),
        ('ECS Load Balancer Configuration', 'Container Port'),
    ]) or source_is(row, [
        ('ECS Container Definition Port Mapping', 'Container Port')
    ]):
        assert_error(row,
            [
                None,
                'Source and destination sockets both have non-identity functions',
            ],
            dest_func=['containerNameToLBConfigContainerName', 'containerPortToLBConfigContainerPort', 'containerPortToOutputSocket']
        )
        return f"FIXED-ENG-3146: unmarked si:identity functions"

    if source_is(row, (['AMI', 'AWS::EC2::AMI'], 'Image ID')):
        assert_error(row, 'Source socket prototype has multiple arguments')
        return f"NEW: AMI/Image ID calls out to AWS and has multiple arguments"

    if dest_is(row, ('Target Group', 'Instance ID')):
        assert_error(row, 'Destination prop Targets passes multiple args to function si:awsTargetGroupNormalizeTargets')
        return f"NEW: Target Group/Instance ID (ELBv2) takes in data from source component *and* destination component"

    # ENG-3147: reaching into payload/code/resource_value
    if source_is(row, [
                        ('Subnet', 'Subnet ID'),
                    ]) and row.get('DestFunction') == 'si:normalizeToArray':
        assert_error(row,
            [
                None,
                'Source and destination sockets both have non-identity functions',
            ],
            source_func=['si:awsSubnetIdFromResource']
        )
        return f"FIXED-ENG-3147: Subnet ID"

    # ENG-3147: reaching into payload/code/resource_value
    if source_is(row, [
                        ('Subnet', 'Subnet ID'),
                        ('Container Definition', 'Container Definition')
                    ]):
        assert_error(row,
            [
                None,
                'Source and destination sockets both have non-identity functions',
            ],
            source_func=['si:awsSubnetIdFromResource', 'si:awsEcsContainerDefGenerated']
        )
        return f"ENG-3147: reaching into payload/code/resource_value"
    
    # ENG-3148: IAM Principal connections
    if source_is(row, (
        [
            'AWS::IAM::AccountPrincipal',
            'AWS IAM Account Principal',
            'AWS::IAM::OIDCSessionPrincipal',
            'AWS IAM OIDC Session Principal',
            'AWS::IAM::ServicePrincipal',
            'AWS IAM AWS Service Principal',
        ],
        'Principal'
    )) and dest_is(row, (
        [
            'AWS::IAM::PolicyStatement',
            'AWS IAM Policy Statement'
        ],
        'Principal'
    )):
        assert_error(row,
            [
                None,
                'Source and destination sockets both have non-identity functions',
                'Multiple connections to the same prop'
            ],
            source_func='awsIamAwsServciePrincipalSetPrincipalOutput',
            dest_func='setPrincipalFromPrincipalSocket'
        )
        return f"ENG-3148: IAM Principal connections with simple subscriptions"

    # ENG-3148: IAM Principal connections
    if source_is(row, (
        [
            'AWS::IAM::AccountPrincipal',
            'AWS IAM Account Principal',
            'AWS::IAM::OIDCSessionPrincipal',
            'AWS IAM OIDC Session Principal',
            'AWS::IAM::PolicyPrincipal',
            'AWS IAM Policy Principal',
            'AWS::IAM::RolePrincipal',
            'AWS IAM Role Principal',
            'AWS::IAM::ServicePrincipal',
            'AWS IAM AWS Service Principal',
            'AWS::IAM::SAMLSessionPrincipal',
            'AWS IAM SAML Session Principal',
            'AWS::IAM::STSFederatedUserPrincipal',
            'AWS IAM STS Federated User Principal',
            'AWS::IAM::UserPrincipal',
            'AWS IAM User Principal',
        ],
        'Principal'
    )) and dest_is(row, (
        [
            'AWS::IAM::PolicyStatement',
            'AWS IAM Policy Statement'
        ],
        'Principal'
    )):
        assert_error(row,
            [
                None,
                'Source and destination sockets both have non-identity functions',
                'Multiple connections to the same prop'
            ],
            source_func='awsIamAwsServciePrincipalSetPrincipalOutput',
            dest_func='setPrincipalFromPrincipalSocket'
        )
        return f"ENG-3148: IAM Principal connections with custom functions"

    # ENG-3158: IAM Condition connections
    if source_is(row, ('AWS::IAM::ConditionOperator', 'Condition')) and dest_is(row, ('AWS::IAM::PolicyStatement', 'Condition')):
        assert_error(row,
            [
                'Source socket prototype has multiple arguments'
            ]
        )
        return f"ENG-3158: IAM Condition connections"

    if source_is(row, (['AWS::IAM::PolicyStatement', 'AWS IAM Policy Statement'], 'Statement')):
        assert dest_is(row, ([
            'AWS ECR Private Repository Policy',
            'AWS::IAM::CustomerManagedIdentityPolicy',
            'AWS::IAM::PolicyDocument',
            'AWS IAM Policy Document',
            'AWS::IAM::Role',
            'AWS IAM Role',
            'Bucket Policy',
            'KMS Key Policy',
        ], 'Statement'))
        assert_error(row,
            [None, 'Source and destination sockets both have non-identity functions'],
            source_func='awsIamPolicyStatementSetStatementOutput',
            dest_func=['s3BucketPolicyStatementsFromSocket', 'Input Socket: Statement', 'kmsKeyPolicyFromStatementSocket', 'Statement Input Socket', 'ecrPrivateRepositoryStatementsFromSocket']
        )
        return f"ENG-3150: IAM Statement connections"

    if source_is(row, [
        ('ECS Load Balancer Configuration', 'Load Balancer Configuration'),
        ('ECS Container Definition Port Mapping', 'Port Mapping')
    ]):
        assert_error(row, 'Source socket prototype has multiple arguments')
        return f"ENG-3149: multi-argument /domain functions"

    return None

def interpret_line(line: str) -> dict[str, str]:
    """Format a line for CSV output."""
    match = re.search(r"^(ERROR (?P<Error>.*) \| )?socket connection (?P<SourceSocket>(.+)) on (?P<SourceName>.+) \((?P<SourceComponentId>\w+)\) --> (?P<DestSocket>(.+)) on (?P<DestName>.+) \((?P<DestComponentId>\w+)\)( \| to prop (?P<SourceProp>.+) --> (?P<DestProp>.+))? \| \((inferred connection|explicit connection APA (?P<ExplicitConnectionApaId>\w+))\)$", line)
    if match is None:
        if line.startswith("ERROR "):
            return {
                'Error': line[6:],
                'ErrorCategory': line[6:],
            }
        else:
            raise ValueError("Failed to parse line: " + line)
    row = match.groupdict()
    row['SourceSchema'], row['SourceComponent'] = interpret_name(row.pop('SourceName'))
    row['DestSchema'], row['DestComponent'] = interpret_name(row.pop('DestName'))
    error = row.get('Error')
    if error is not None:
        match = re.search(r"^(?P<Error>Source and destination sockets both have non-identity functions): (?P<DestFunction>.+) and (?P<SourceFunction>.+)$", error)
        if match is not None:
            row['SourceFunction'] = match.group('SourceFunction')
            row['DestFunction'] = match.group('DestFunction')
            row['Error'] = match.group('Error')
    row['ErrorCategory'] = category_of(row)
    return row

def interpret_name(name: str) -> tuple[str, str]:
    match = re.search(r"^(\S+::\S+) (.+)$", name)
    if match is not None:
        return match.group(1), match.group(2)
    match = re.search(r"^(.+) (\S*)$", name)
    if match is not None:
        return match.group(1), match.group(2)
    else:
        return name, ''

# deleted_workspaces = read_workspace_list('../deleted_workspaces.txt')
# only_workspaces = read_workspace_list('../workspaces.txt')

for file in os.listdir('.'):
    if file.endswith('.txt'):
        match = re.search(r'^(.+)\.(\w+)\.(\w+)\.txt$', file)
        if match is None:
            raise ValueError(f"File {file} does not match expected pattern")

        workspace_name, workspace_id, change_set_id = match.groups()
        # if deleted_workspaces and workspace_id in deleted_workspaces:
        #     continue

        # Read the text file
        with open(file, 'r') as textfile:
            rows = [interpret_line(line) for line in textfile.read().splitlines()]
        csv_filename = f"{workspace_name}.{workspace_id}.{change_set_id}.csv"
        print(f"{csv_filename}")
        with open(csv_filename, 'w') as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=[
                'DestSchema', 'DestSocket', 'SourceSchema', 'SourceSocket',
                'DestFunction', 'SourceFunction',
                'ErrorCategory',
                'Error',
                'ExplicitConnectionApaId',
                'SourceProp', 'DestProp',
                'DestComponentId', 'DestComponent', 'SourceComponentId', 'SourceComponent',
                'WorkspaceId', 'ChangeSetId', 'WorkspaceName'
            ])
            writer.writeheader()
            writer.writerows(rows)
