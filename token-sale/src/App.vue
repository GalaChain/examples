<template>
  <TopNav />
  <main>
    <RouterView />
  </main>
</template>

<script setup lang="ts">
  import { RouterView } from 'vue-router';
  import TopNav from './components/shared/TopNav.vue';
</script>

<style scoped>
</style>
