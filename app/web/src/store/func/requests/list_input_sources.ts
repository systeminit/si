import { PropKind } from "@/api/sdf/dal/prop";
import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";

export interface InputSourceSocket {
  schemaVariantId: string;
  internalProviderId: string;
  name: string;
}

export interface InputSourceProp {
  propId: string;
  kind: PropKind;
  schemaVariantId: string;
  internalProviderId?: string;
  path: string;
  name: string;
}

export interface ListInputSourcesResponse {
  sockets: InputSourceSocket[];
  props: InputSourceProp[];
}

export const listInputSources = (
  visibility: Visibility,
  onSuccess: (response: ListInputSourcesResponse) => void,
) =>
  new ApiRequest<ListInputSourcesResponse, Visibility>({
    url: "func/list_input_sources",
    params: { ...visibility },
    onSuccess,
  });
