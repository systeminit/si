import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { GlobalErrorService } from "@/service/global_error";
import { PropKind } from "@/api/sdf/dal/prop";
import { ApiResponse } from "@/api/sdf";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";

export interface InputSourceSocket {
  schemaVariantId: number;
  internalProviderId: number;
  name: string;
}

export interface InputSourceProp {
  propId: number;
  kind: PropKind;
  schemaVariantId: number;
  internalProviderId: number;
  path: string;
  name: string;
}

export interface ListInputSourcesResponse {
  sockets: InputSourceSocket[];
  props: InputSourceProp[];
}

const memo: {
  [key: string]: Observable<ListInputSourcesResponse>;
} = {};

export const listInputSources: () => Observable<ListInputSourcesResponse> =
  memoizedVisibilitySdfPipe(
    (visibility, sdf) =>
      sdf
        .get<ApiResponse<ListInputSourcesResponse>>("func/list_input_sources", {
          ...visibility,
        })
        .pipe(
          map((response) => {
            if (response.error) {
              GlobalErrorService.set(response);
              return { sockets: [], props: [] };
            }

            return response as ListInputSourcesResponse;
          }),
        ),
    memo,
  );
