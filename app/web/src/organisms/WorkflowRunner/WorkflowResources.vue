<template>
  <div
    v-if="components === undefined"
    class="w-full h-full p-md text-center text-lg"
  >
    No Resources Impacted
  </div>
  <div v-else class="w-full h-full">
    <template v-for="component in components" :key="component.id">
      <WorkflowResource
        v-if="component.resource"
        :name="component.name"
        :kind="component.resource.kind"
        :output="whiskersJson"
        :status="component.resource.health || 'Unknown'"
        :default-closed="defaultClosed"
        :component="component"
      />
    </template>
  </div>
  <!-- <WorkflowResource
    name="joe"
    kind="aws region"
    output="whatever"
    status="Ok"
    :default-closed="defaultClosed"
  />
  <WorkflowResource
    name="bob"
    kind="aws instance"
    output="whatever"
    status="Error"
    :default-closed="defaultClosed"
  />
  <WorkflowResource
    name="ann"
    kind="other resource"
    output="whatever"
    status="Unknown"
    :default-closed="defaultClosed"
  /> -->
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import WorkflowResource from "./WorkflowResource.vue";
import { ComponentListItem } from "../StatusBar/StatusBarTabPanelComponentList.vue";

defineProps({
  components: { type: Array as PropType<ComponentListItem[]> },
  defaultClosed: { type: Boolean, default: true },
});

const whiskersJson = `[
{
  "Id": "sha256:2e9d848965aaa71f9024e42267f42b1dff73092ccb084b1e4d619bab718cd864",
  "RepoTags": [
      "systeminit/whiskers:latest"
  ],
  "RepoDigests": [
      "systeminit/whiskers@sha256:bbf3d758427f312f157660b69c20f5f0e9e870cb471bd8a62fda6f908a9b7d47"
  ],
  "Parent": "",
  "Comment": "",
  "Created": "2020-08-12T18:09:50.000243049Z",
  "Container": "",
  "ContainerConfig": {
      "Hostname": "",
      "Domainname": "",
      "User": "",
      "AttachStdin": false,
      "AttachStdout": false,
      "AttachStderr": false,
      "ExposedPorts": {
          "80/tcp": {}
      },
      "Tty": false,
      "OpenStdin": false,
      "StdinOnce": false,
      "Env": [
          "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
          "NGINX_VERSION=1.19.1",
          "NJS_VERSION=0.4.2",
          "PKG_RELEASE=1"
      ],
      "Cmd": [
          "/bin/sh",
          "-c",
          "#(nop) COPY dir:b3aa00c72d66a99b5def5676eaaeece4620d9545d68cd83870e619e2cdd7336f in /usr/share/nginx/html "
      ],
      "ArgsEscaped": true,
      "Image": "sha256:ecd67fe340f9048eaf01f304a37e9be9298480f242cc3e332a8a84c754df7d9a",
      "Volumes": null,
      "WorkingDir": "",
      "Entrypoint": [
          "/docker-entrypoint.sh"
      ],
      "OnBuild": null,
      "Labels": {
          "maintainer": "NGINX Docker Maintainers <docker-maint@nginx.com>"
      },
      "StopSignal": "SIGTERM"
  },
  "DockerVersion": "19.03.12+azure",
  "Author": "",
  "Config": {
      "Hostname": "",
      "Domainname": "",
      "User": "",
      "AttachStdin": false,
      "AttachStdout": false,
      "AttachStderr": false,
      "ExposedPorts": {
          "80/tcp": {}
      },
      "Tty": false,
      "OpenStdin": false,
      "StdinOnce": false,
      "Env": [
          "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
          "NGINX_VERSION=1.19.1",
          "NJS_VERSION=0.4.2",
          "PKG_RELEASE=1"
      ],
      "Cmd": [
          "nginx",
          "-g",
          "daemon off;"
      ],
      "ArgsEscaped": true,
      "Image": "sha256:ecd67fe340f9048eaf01f304a37e9be9298480f242cc3e332a8a84c754df7d9a",
      "Volumes": null,
      "WorkingDir": "",
      "Entrypoint": [
          "/docker-entrypoint.sh"
      ],
      "OnBuild": null,
      "Labels": {
          "maintainer": "NGINX Docker Maintainers <docker-maint@nginx.com>"
      },
      "StopSignal": "SIGTERM"
  },
  "Architecture": "amd64",
  "Os": "linux",
  "Size": 23742233,
  "VirtualSize": 23742233,
  "GraphDriver": {
      "Data": {
          "LowerDir": "/var/lib/docker/overlay2/17554b0115273daceb99c7f27689359a368c98e58d1d7971cdcdb6046a03d9e2/diff:/var/lib/docker/overlay2/b0921f7ea216a76257e81f4baaa75274859349d959092b5b68b72cd8c9682f2c/diff:/var/lib/docker/overlay2/6d235e4abedc185bddbc12ef951f8494508309005591cfc203a99927ddcdeee8/diff:/var/lib/docker/overlay2/79737812f91d5c096459c18d2767dcf1d8ccba7fd167968723151a319671c1cd/diff:/var/lib/docker/overlay2/dce36d52cf20dbf32a53d4a08aa0b910cff6259b3ef18c67f23af0e4419e59fe/diff",
          "MergedDir": "/var/lib/docker/overlay2/6e5b39f319b82feddcda373ef3e5dd94a8a1b51ac80098c11efacdfc54654174/merged",
          "UpperDir": "/var/lib/docker/overlay2/6e5b39f319b82feddcda373ef3e5dd94a8a1b51ac80098c11efacdfc54654174/diff",
          "WorkDir": "/var/lib/docker/overlay2/6e5b39f319b82feddcda373ef3e5dd94a8a1b51ac80098c11efacdfc54654174/work"
      },
      "Name": "overlay2"
  },
  "RootFS": {
      "Type": "layers",
      "Layers": [
          "sha256:3e207b409db364b595ba862cdc12be96dcdad8e36c59a03b7b3b61c946a5741a",
          "sha256:5f1add6e505be671525b6ccfe5e0dd4f269d72fec5152594499dbb91ea33c7db",
          "sha256:08fb2e2ff084bb8353bbc4181aad256bf601377b7d4fa2c79fdf194b7f92a831",
          "sha256:b7d86c86e4326f1f5823b2ed68faf0354abafbb7099ce71be92c31e3ddfc21a9",
          "sha256:689cc6c05bc7bd66c3dc3bd0034dc910d8c21954cb719f9c9eae5a1a5888102b",
          "sha256:ea4e4b884d9d7938ea2b3418a1603713db5c38a0c1830016b16f9dbd398b605b"
      ]
  },
  "Metadata": {
      "LastTagTime": "0001-01-01T00:00:00Z"
  }
}
]`;
</script>
