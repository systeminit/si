---
outline:
  level: [2, 3, 4]
---

# How to use the Public API to create a component

This how-to assumes:

- Basic [familiarity with System Initiative](../tutorials/index.md)
- Have generated a
  [workspace scoped API token](../explanation/generate-a-workspace-api-token)
- Access to an AWS account

It will teach you how to use the
[System Initiative Public API](../reference/public-api) to manage your
infrastructure.

We will cover:

- Creating a change set
- Creating an AWS Credential and Region component
- Create a Standard AWS VPC using our template

## Walkthrough

We are going to build a python application that uses the Public API. The full
code will be available at the end of the guide.

### Set the correct environment variables

The application we are going to write uses 2 environment variables:

- SI_API_TOKEN
- SI_WORKSPACE_ID

These are required and the application will fail without them

### Create a Python Virtual Environment

As you will be installing a project dependency, let's create a Python virtal
environment. Open your terminal and run the following code:

```shell
mkdir demo-application
cd demo-application
python3 -m venv venv
source venv/bin/activate
```

Now you can install the dependency that we need:

```shell
pip install requests
```

### Create the basic structure of our python application

The basic structure of the python application is as follows:

```python
import os
import requests

API_TOKEN = os.environ.get('SI_API_TOKEN')
WORKSPACE_ID = os.environ.get('SI_WORKSPACE_ID')
API_URL="https://api.systeminit.com"

headers = {
    'Authorization': f'Bearer {API_TOKEN}',
    'Content-Type': 'application/json'
}

def main():
    try:
        print('Getting started with the System Initiative Public API')

    except requests.exceptions.HTTPError as err:
        print(f'HTTP Error: {err}')
        print(f'Response: {err.response.text}')
    except Exception as err:
        print(f'Error: {err}')

if __name__ == '__main__':
 main()

```

Write that code to `main.py` and you can execute that code as follows:

```shell
python main.py
```

If things are configured correctly, then you will get the following output:

```shell
python main.py
Getting started with the System Initiative Public API
(venv)
```

### Creating a change set

All changes in System Initiative happen in a change set so the first segment of
code we will write is being able to create a change set. Lets create a function
that we can use in our application to create a change set

```python
def create_change_set(name):
    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets',
        headers=headers,
        json={'changeSetName': name}
    )

    response.raise_for_status()
    return response.json()
```

You can use that code in your application as follows:

```python
def main():
    try:
        print('Getting started with the System Initiative Public API')

        print('Creating change set...')
        change_set_data = create_change_set("how-to-guide")
        change_set_id = change_set_data["changeSet"]["id"]
        print(f'Change set ID: {change_set_id}')

    except requests.exceptions.HTTPError as err:
        print(f'HTTP Error: {err}')
        print(f'Response: {err.response.text}')
    except Exception as err:
        print(f'Error: {err}')
```

Executing the code gives us the following:

```shell
python main.py
Getting started with the System Initiative Public API
1. Creating change set...
Change set ID: 01JWATEG60KHXWC4N1J3NG8A1X
(venv)
```

Each time that application is executed, a new change set will be created called
'how-to-guide'.

### Creating an AWS Credential

When creating an `AWS Credential` component, there are 3 operations that need to
be carried out:

1. Create the AWS component
2. Create the secret
3. Update the component with the secret

We can build the functions as follows:

```python
def create_component(change_set_id, schema_name, name, options=None):
    request_body = {
        'schemaName': schema_name,
        'name': name
    }

    if options:
        request_body.update(options)

    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components',
        headers=headers,
        json=request_body
    )

    response.raise_for_status()
    return response.json()

def create_secret(change_set_id, name, definition_name, raw_data):
    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/secrets',
        headers=headers,
        json={
            'name': name,
            'definitionName': definition_name,
            'description': f'Secret for {name}',
            'rawData': raw_data
        }
    )

    response.raise_for_status()
    return response.json()

def update_component(change_set_id, component_id, options=None):
    request_body = {}

    if options:
        request_body.update(options)

    response = requests.put(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/{component_id}',
        headers=headers,
        json=request_body
    )

    response.raise_for_status()
    return response.json()
```

These functions allow us to build out the rest of our application. So lets use
them in our main function:

```python
def main():
    try:
        print('Getting started with the System Initiative Public API')

        print('1. Creating change set...')
        change_set_data = create_change_set("how-to-guide")
        change_set_id = change_set_data["changeSet"]["id"]
        print(f'Change set ID: {change_set_id}')

        print('2. Creating AWS Credential')
        credential_component_data = create_component(change_set_id,
            "AWS Credential",  # SchemaName
            "demo-credential", # ComponentName
            None)
        credential_component_id = credential_component_data["component"]["id"]
        print(f'AWS Credential created with ID: {credential_component_id}')

        print('3. Creating secret using AWS account')
        create_secret(change_set_id,
                "demo-credential", # SecretName
                "AWS Credential",  # SecretDefinitionName
                {
                    # FIXME: set environment variables and read from them here to
                    # ensure that we don't hard code credentials in our application
                    "AccessKeyId": "<your-access-key-id-via-environment-variable>",
                    "SecretAccessKey": "<your-secret-access-key-via-environment-variable>",
                    "SessionToken": "<your-session-token-via-environment-variable>"
                })

        print('4. Setting secret to AWS Credential component')
        update_component(change_set_id, credential_component_id, {
            'secrets': {
                'AWS Credential': 'demo-credential'
                # This will set the `AWS Credential` secret prop to the `demo-credential` secret
            }
        })

    except requests.exceptions.HTTPError as err:
        print(f'HTTP Error: {err}')
        print(f'Response: {err.response.text}')
    except Exception as err:
        print(f'Error: {err}')
```

Executing that application now gives us:

```shell
python main.py
Getting started with the System Initiative Public API
1. Creating change set...
Change set ID: 01JWATWKKQZQEN7KGWAXNZRSEA
2. Creating AWS Credential
AWS Credential created with ID: 01JWATWM7NR32CK2ATRDYRXM63
3. Creating secret using AWS account
4. Setting secret to AWS Credential component
(venv)
```

If we go to our change set in our workspace, we can now see that the AWS
credential has been created and the qualification has successfully run if you
have provided the correct credentials.

### Creating a region

Based on the functions we created above, we can now create a `Region` component
and create a connection from that component to the `AWS Credential` component.

You can enhance the data that you pass to the create_component function to
include connections data:

```python
        print('5. Creating Region')
        region_options = {
            "attributes": {
                "/domain/region": "us-east-1",
                "/secrets/credential": {
                    "$source": {
                        "component": "demo-credential",
                        "path": "/secrets/AWS Credential"
                    }
                }
            }
        }
        region_component_data = create_component(change_set_id,
            "Region",     # SchemaName
            "us-east-1",  # ComponentName
            region_options)
        region_component_id = region_component_data["component"]["id"]
        print(f'Region created with ID: {region_component_id}')
```

### Use the AWS standard VPC template

System Initiative provides a template that can be used to configure an
[AWS VPC](https://docs.aws.amazon.com/vpc/). We can use this template component,
make the connection to the region and the credential and set the correct
properties required to expand the component parts of the template.

```python
        print('6. Creating AWS VPC Template')
        template_options = {
            "attributes": {
                "/domain/VPC Name": "demo-from-api",
                "/secrets/AWS Credential": {
                    "$source": {
                        "component": "demo-credential",
                        "path": "/secrets/AWS Credential"
                    }
                },
                "/domain/extra/Region": {
                    "$source": {
                        "component": "us-east-1",
                        "path": "/domain/region"
                    }
                }
            }
        }
        template_component_data = create_component(change_set_id,
            "AWS VPC Template",    # SchemaName
            "demo-template",       # ComponentName
            template_options)
        template_component_id = template_component_data["component"]["id"]
        print(f'Template Component created with ID: {template_component_id}')
```

### Expanding the template

A template will have a management function attached to it that can be executed.
Executing this function will create all the component parts of the VPC and make
all the appropriate connections between these component parts and the region and
credential components.

We need to create 1 new function to be able to do that:

```python
def execute_management_function(change_set_id, component_id, options=None):
    request_body = {}

    if options:
        request_body.update(options)

    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/{component_id}/execute-management-function',
        headers=headers,
        json=request_body
    )

    response.raise_for_status()
    return response.json()
```

We can use this function as follows:

```python
        print('7. Expanding Template')
        expansion_options = {
            "managementFunction": {
                "function": "Run Template"
                ## This is the name of the management function attached to the template
            }
        }
        func_run_data = execute_management_function(change_set_id, template_component_id, expansion_options)
        management_func_job_state_id = func_run_data["managementFuncJobStateId"]
        print(f'Template Func running with management func job state ID: {management_func_job_state_id}')
```

When we execute the application, we get the output as follows:

```shell
python main.py
Getting started with the System Initiative Public API
1. Creating change set...
Change set ID: 01JWAVDGMEDYK1TRJJVQMQ6ACN
2. Creating AWS Credential
AWS Credential created with ID: 01JWAVDH83NJWJ1K2X5DXWEQ8M
3. Creating secret using AWS account
4. Setting secret to AWS Credential component
5. Creating Region
Region created with ID: 01JWAVDK6T7TP47H33PJZFXA90
6. Creating AWS Standard VPC Template
Template Component created with ID: 01JWAVDKYRK5CEJ3XCT8F55DXC
7. Expanding Template
Template Func running with management func job state ID: 01JWAVDST3701864MFYQMXR4CH
(venv)
```

We have created the `Region`, `AWS Credential` and `AWS Standard VPC Template`
components and then expanded that template to create the component parts that
represent the pieces of the VPC. Each of the component parts have all of the
connections they need to the credential and the region. It has also queued all
of the create actions required to deploy the VPC infrastructure.

### Python code

```python
import os
import requests

API_TOKEN = os.environ.get('SI_API_TOKEN')
WORKSPACE_ID = os.environ.get("SI_WORKSPACE_ID")
API_URL="https://api.systeminit.com"

headers = {
    'Authorization': f'Bearer {API_TOKEN}',
    'Content-Type': 'application/json'
}

def create_change_set(name):
    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets',
        headers=headers,
        json={'changeSetName': name}
    )

    response.raise_for_status()
    return response.json()

def create_component(change_set_id, schema_name, name, options=None):
    request_body = {
        'schemaName': schema_name,
        'name': name
    }

    if options:
        request_body.update(options)

    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components',
        headers=headers,
        json=request_body
    )

    response.raise_for_status()
    return response.json()

def update_component(change_set_id, component_id, options=None):
    request_body = {}

    if options:
        request_body.update(options)

    response = requests.put(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/{component_id}',
        headers=headers,
        json=request_body
    )

    response.raise_for_status()
    return response.json()

def execute_management_function(change_set_id, component_id, options=None):
    request_body = {}

    if options:
        request_body.update(options)

    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/{component_id}/execute-management-function',
        headers=headers,
        json=request_body
    )

    response.raise_for_status()
    return response.json()

def create_secret(change_set_id, name, definition_name, raw_data):
    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/secrets',
        headers=headers,
        json={
            'name': name,
            'definitionName': definition_name,
            'description': f'Secret for {name}',
            'rawData': raw_data
        }
    )

    response.raise_for_status()
    return response.json()

def main():
    try:
        print('Getting started with the System Initiative Public API')

        print('1. Creating change set...')
        change_set_data = create_change_set("how-to-guide")
        change_set_id = change_set_data["changeSet"]["id"]
        print(f'Change set ID: {change_set_id}')

        print('2. Creating AWS Credential')
        credential_component_data = create_component(change_set_id, "AWS Credential", "demo-credential", None)
        credential_component_id = credential_component_data["component"]["id"]
        print(f'AWS Credential created with ID: {credential_component_id}')

        print('3. Creating secret using AWS account')
        create_secret(change_set_id,
                "demo-credential",
                "AWS Credential",
                {
                    # FIXME: set environment variables and read from them here to
                    # ensure that we don't hard code credentials in our application
                    "AccessKeyId": "<your-access-key-id-via-environment-variable>",
                    "SecretAccessKey": "<your-secret-access-key-via-environment-variable>",
                    "SessionToken": "<your-session-token-via-environment-variable>"
                })

        print('4. Setting secret to AWS Credential component')
        update_component(change_set_id, credential_component_id, {
            'secrets': {
                'AWS Credential': 'demo-credential'
            }
        })

        print('5. Creating Region')
        region_options = {
            "attributes": {
                "/domain/region": "us-east-1",
                "/secrets/credential": {
                    "$source": {
                        "component": "demo-credential",
                        "path": "/secrets/AWS Credential"
                    }
                }
            }
        }
        region_component_data = create_component(change_set_id, "Region", "us-east-1", region_options)
        region_component_id = region_component_data["component"]["id"]
        print(f'Region created with ID: {region_component_id}')

        print('6. Creating AWS VPC Template')
        template_options = {
            "attributes": {
                "/domain/VPC Name": "paul-demo-from-api",
                "/secrets/AWS Credential": {
                    "$source": {
                        "component": "demo-credential",
                        "path": "/secrets/AWS Credential"
                    }
                },
                "/domain/extra/Region": {
                    "$source": {
                        "component": "us-east-1",
                        "path": "/domain/region"
                    }
                }
            }
        }
        template_component_data = create_component(change_set_id, "AWS VPC Template", "demo-template", template_options)
        template_component_id = template_component_data["component"]["id"]
        print(f'Template Component created with ID: {template_component_id}')

        print('7. Expanding Template')
        expansion_options = {
            "managementFunction": {
                "function": "Run Template"
            }
        }
        func_run_data = execute_management_function(change_set_id, template_component_id, expansion_options)
        management_func_job_state_id = func_run_data["managementFuncJobStateId"]
        print(f'Template Func running with management func job state ID: {management_func_job_state_id}')

    except requests.exceptions.HTTPError as err:
        print(f'HTTP Error: {err}')
        print(f'Response: {err.response.text}')
    except Exception as err:
        print(f'Error: {err}')

if __name__ == '__main__':
 main()

```

### TypeScript code

```typescript
const API_TOKEN = process.env.SI_API_TOKEN;
const WORKSPACE_ID = process.env.SI_WORKSPACE_ID;
const API_URL = "https://api.systeminit.com";

const headers = {
  Authorization: `Bearer ${API_TOKEN}`,
  "Content-Type": "application/json",
};

interface ChangeSetResponse {
  changeSet: {
    id: string;
  };
}

interface ComponentResponse {
  component: {
    id: string;
  };
}

interface FuncRunResponse {
  funcRunId: string;
}

interface ComponentOptions {
  domain?: Record<string, any>;
  connections?: Array<{
    from: {
      component: string;
      socketName: string;
    };
    to: string;
  }>;
  secrets?: Record<string, string>;
}

interface ManagementFunctionOptions {
  managementFunction: {
    function: string;
  };
}

async function createChangeSet(name: string): Promise<ChangeSetResponse> {
  const response = await fetch(`${API_URL}/v1/w/${WORKSPACE_ID}/change-sets`, {
    method: "POST",
    headers,
    body: JSON.stringify({
      changeSetName: name,
    }),
  });

  if (!response.ok) {
    throw new Error(`HTTP Error: ${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<ChangeSetResponse>;
}

async function createComponent(
  changeSetId: string,
  schemaName: string,
  name: string,
  options?: ComponentOptions,
): Promise<ComponentResponse> {
  const requestBody: any = {
    schemaName,
    name,
  };

  if (options) {
    Object.assign(requestBody, options);
  }

  const response = await fetch(
    `${API_URL}/v1/w/${WORKSPACE_ID}/change-sets/${changeSetId}/components`,
    {
      method: "POST",
      headers,
      body: JSON.stringify(requestBody),
    },
  );

  if (!response.ok) {
    throw new Error(`HTTP Error: ${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<ComponentResponse>;
}

async function updateComponent(
  changeSetId: string,
  componentId: string,
  options?: ComponentOptions,
): Promise<any> {
  const requestBody: any = {};

  if (options) {
    Object.assign(requestBody, options);
  }

  const response = await fetch(
    `${API_URL}/v1/w/${WORKSPACE_ID}/change-sets/${changeSetId}/components/${componentId}`,
    {
      method: "PUT",
      headers,
      body: JSON.stringify(requestBody),
    },
  );

  if (!response.ok) {
    throw new Error(`HTTP Error: ${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<any>;
}

async function executeManagementFunction(
  changeSetId: string,
  componentId: string,
  options?: ManagementFunctionOptions,
): Promise<FuncRunResponse> {
  const requestBody: any = {};

  if (options) {
    Object.assign(requestBody, options);
  }

  const response = await fetch(
    `${API_URL}/v1/w/${WORKSPACE_ID}/change-sets/${changeSetId}/components/${componentId}/execute-management-function`,
    {
      method: "POST",
      headers,
      body: JSON.stringify(requestBody),
    },
  );

  if (!response.ok) {
    throw new Error(`HTTP Error: ${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<FuncRunResponse>;
}

async function createSecret(
  changeSetId: string,
  name: string,
  definitionName: string,
  rawData: Record<string, any>,
): Promise<any> {
  const response = await fetch(
    `${API_URL}/v1/w/${WORKSPACE_ID}/change-sets/${changeSetId}/secrets`,
    {
      method: "POST",
      headers,
      body: JSON.stringify({
        name,
        definitionName,
        description: `Secret for ${name}`,
        rawData,
      }),
    },
  );

  if (!response.ok) {
    throw new Error(`HTTP Error: ${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<any>;
}

async function main(): Promise<void> {
  try {
    console.log("Creating change set...");
    const changeSetData = await createChangeSet("how-to-api");
    const changeSetId = changeSetData.changeSet.id;
    console.log(`Change set ID: ${changeSetId}`);

    console.log("Creating AWS Credential");
    const credentialComponentData = await createComponent(
      changeSetId,
      "AWS Credential",
      "demo-credential",
    );
    const credentialComponentId = credentialComponentData.component.id;
    console.log(`AWS Credential created with ID: ${credentialComponentId}`);

    console.log("Creating secret using AWS account");
    await createSecret(changeSetId, "demo-credential", "AWS Credential", {
      // FIXME: set environment variables and read from them here to
      // ensure that we don't hard code credentials in our application
      "AccessKeyId": "<your-access-key-id-via-environment-variable>",
      "SecretAccessKey": "<your-secret-access-key-via-environment-variable>",
      "SessionToken": "<your-session-token-via-environment-variable>"
    });

    console.log("4. Setting secret to AWS Credential component");
    await updateComponent(changeSetId, credentialComponentId, {
      secrets: {
        "AWS Credential": "demo-credential",
      },
    });

    console.log("5. Creating Region");
    const regionOptions: ComponentOptions = {
      attributes: {
        "/domain/region": "us-east-1",
        "/secrets/credential": {
          $source: {
            component: "demo-credential",
            path: "/secrets/AWS Credential",
          },
        },
      },
    };
    const regionComponentData = await createComponent(
      changeSetId,
      "Region",
      "us-east-1",
      regionOptions,
    );
    const regionComponentId = regionComponentData.component.id;
    console.log(`Region created with ID: ${regionComponentId}`);

    console.log("6. Creating AWS Standard VPC Template");
    const templateOptions: ComponentOptions = {
      attributes: {
        "/domain/VPC Name": "paul-demo-from-api",
        "/secrets/AWS Credential": {
          $source: {
            component: "demo-credential",
            path: "/secrets/AWS Credential",
          },
        },
        "/domain/extra/Region": {
          $source: {
            component: "us-east-1",
            path: "/domain/region",
          },
        },
      },
    };
    const templateComponentData = await createComponent(
      changeSetId,
      "AWS VPC Template",
      "demo-template",
      templateOptions,
    );
    const templateComponentId = templateComponentData.component.id;
    console.log(`Template Component created with ID: ${templateComponentId}`);

    console.log("7. Expanding Template");
    const expansionOptions: ManagementFunctionOptions = {
      managementFunction: {
        function: "Run Template",
      },
    };
    const funcRunData = await executeManagementFunction(
      changeSetId,
      templateComponentId,
      expansionOptions,
    );
    const managementFuncJobStateId = funcRunData.managementFuncJobStateId;
    console.log(
      `Template Func running with management func job state ID: ${managementFuncJobStateId}`,
    );
  } catch (error: any) {
    if (error.message.includes("HTTP Error")) {
      console.log(`HTTP Error: ${error.message}`);
    } else {
      console.log(`Error: ${error.message}`);
    }
  }
}

main();
```
