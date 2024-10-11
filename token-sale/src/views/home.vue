<template>
<div class="container">
  <form @submit.prevent="goToSale">
    <h1>Enter Token Sale ID</h1>
    <input type="text" placeholder="Token Sale ID" v-model="saleId" />
    <button 
      type="submit"
      :disabled="!saleId"
      >
          View Sale
    </button>
  </form>
</div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
const router = useRouter();

const saleId = ref<string>();
const goToSale = () => {
  router.push(`/sale/${saleId.value}`);
}
</script>

<style scoped>
.container {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

input {
  max-width: 30rem;
  width:100%;
}

button {
  margin-top: 2rem;
}
</style>