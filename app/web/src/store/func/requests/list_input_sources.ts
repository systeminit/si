import { PropKind } from "@/api/sdf/dal/prop";
import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";

export interface InputSourceSocket {
  schemaVariantId: number;
  internalProviderId: number;
  name: string;
}

export interface InputSourceProp {
  propId: number;
  kind: PropKind;
  schemaVariantId: number;
  internalProviderId?: number;
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
