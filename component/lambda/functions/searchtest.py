import time
from typing import Optional, cast
from si_types import ChangeSetId, ComponentId, WorkspaceId
from si_luminork_api import SiLuminorkApi
import os
import sys

if len(sys.argv) < 4:
    raise Exception("Usage: searchtest.py <attr_name> <attr_value> <search_method> ...")
attr_name = sys.argv[1]
attr_value = sys.argv[2]
# Get the remaining args
search_methods = sys.argv[3:]

SI_API_URL = os.getenv("SI_API_URL", "http://localhost:5380")
SI_API_TOKEN = os.getenv("SI_API_TOKEN")
if not SI_API_TOKEN:
    raise Exception("SI_API_TOKEN must be set")
SI_WORKSPACE_ID = cast(Optional[WorkspaceId], os.getenv("SI_WORKSPACE_ID"))
if not SI_WORKSPACE_ID:
    raise Exception("SI_WORKSPACE_ID must be set")
SI_CHANGE_SET_ID = cast(Optional[ChangeSetId], os.getenv("SI_CHANGE_SET_ID"))
if not SI_CHANGE_SET_ID:
    raise Exception("SI_CHANGE_SET_ID must be set")

api = SiLuminorkApi(SI_API_URL, SI_API_TOKEN, SI_WORKSPACE_ID, SI_CHANGE_SET_ID)

# for i in range(998):
#     component_name = f"test-component-{i:03}"
#     print(f"Creating component {component_name} ...")
#     component_id = api.create_component("AWS::EC2::Instance", component_name)

# print(api.get_component(cast(ComponentId, "01K6BWP9X9B1Q0HSZAYVMYF3C1")))
best_elapsed = dict[str, int]()
for search_method in search_methods:
    best_elapsed[search_method] = 10000000
    for i in range(100):
        start_time = time.perf_counter()
        component_ids = api.search_spike(attr_name, attr_value, search_method)
        # Truncate to integer
        elapsed = int((time.perf_counter() - start_time) * 1000)
        if len(component_ids) == 0:
            raise Exception("No components found")
        if elapsed < best_elapsed[search_method]:
            best_elapsed[search_method] = elapsed
        print(f"[{search_method}] attempt #{i}: {elapsed} milliseconds")
    print(f"[{search_method}] {best_elapsed[search_method]} milliseconds")
# print(f"Results: {component_ids}")

print("")
print("TOTALS")
for search_method, elapsed in best_elapsed.items():
    print(f"[{search_method}]: {elapsed} milliseconds")