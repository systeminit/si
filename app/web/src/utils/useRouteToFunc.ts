import { useRoute, useRouter } from "vue-router";

export const useRouteToFunc = () => {
  const route = useRoute();
  const router = useRouter();
  return (funcId?: string) => {
    router.push({
      name: "workspace-lab",
      params: {
        ...route.params,
        funcId,
      },
    });
  };
};
