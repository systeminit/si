import { useRoute, useRouter } from "vue-router";

export const useRouteToFunc = () => {
  const route = useRoute();
  const router = useRouter();
  return (funcId?: string) => {
    router.replace({
      name: "workspace-lab-functions",
      params: {
        ...route.params,
        funcId,
      },
    });
  };
};
