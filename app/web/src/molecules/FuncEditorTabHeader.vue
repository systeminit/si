<template>
  <SiTabHeader v-for="(funcId, index) in funcList" :key="funcId">
    {{ editingFuncs[index].origFunc.name }}
    <button class="inline-block rounded-sm" @click="closeFunc(funcId)">
      <VueFeather type="x-circle" />
    </button>
  </SiTabHeader>
</template>
