# system_initiative_api_client

Python SDK for the System Initiative Public API

## Installation

You can install the package via pip:

```bash
pip install system_initiative_api_client
```

## Requirements

Python >=3.8

## Usage

Please refer to the [documentation](https://github.com/systeminit/si) for more information.

### Authentication

This API uses BASIC authentication.

```python
import system_initiative_api_client
from system_initiative_api_client.api_client import ApiClient
from system_initiative_api_client.configuration import Configuration

# Configure API key authorization
api_token = os.environ.get('SI_API_TOKEN')
api_client = system_initiative_api_client.ApiClient(configuration)
api_client.default_headers['Authorization'] = f"Bearer {api_token}"

change_sets_api = ChangeSetsApi(api_client)
workspace_id = os.environ.get("SI_WORKSPACE_ID")

def print_response(response, title="Response"):
    if hasattr(response, "to_dict"):
        response_dict = response.to_dict()
        print(json.dumps(response_dict, indent=2, default=str))

# Example API client usage
list_response = change_sets_api.list_change_sets(workspace_id=workspace_id)
print_response(list_response, "List Change Sets Response")
```

## License

[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0.html)

## Author Information

- **System Initiative** - support@systeminit.com
- **System Initiative** - info@systeminit.com
- **Organization**: System Initiative - https://systeminit.com

## Development

For development, clone this repository and install in development mode:

```bash
git clone https://github.com/systeminit/si
cd generated-sdks/python
pip install -e .
```
