<template>
  <div class="container">
    <div v-if="loading">
      loading...
    </div>
    <div v-else-if="!sale">
      <h1>Sale not found</h1>
    </div>
    <div v-else>
      <h1>Token Sale</h1>
      <h2>Token Sale ID: {{sale.tokenSaleId}}</h2>
      <h2>Created: {{sale.created}}</h2>
      <h2>Tx ID: {{sale.txId}}</h2>
    </div>
  </div>
  </template>
  
  <script setup lang="ts">
  import { onMounted, ref } from 'vue';
  import { useRoute } from 'vue-router';
import { useConnect } from '../composables/connect';
import { FetchTokenClassesDto, TokenClass } from '@gala-chain/api';
import { plainToInstance } from 'class-transformer';

  const route = useRoute();
  const connect = useConnect();
  const loading = ref<boolean>(true);
  const sale = ref<any>();
  const costTokens = ref<TokenClass[]>([]);
  const saleTokens = ref<TokenClass[]>([]);
  
  onMounted(async () => {
    if(route.params.tokenSaleId) {
      await getTokenSale(route.params.tokenSaleId as string);
    } else {
      loading.value = false;
    }
  })

  const getTokenSale = async (tokenSaleId: string) => {
    try { 
      loading.value = true;
      sale.value = await connect.fetchTokenSaleById(tokenSaleId);
      const costTokenDto = plainToInstance(FetchTokenClassesDto, {tokenClasses: sale.value.cost.map((c:any) => c.tokenClassKey)});
      costTokens.value = await connect.fetchTokenClasses(costTokenDto);
      const saleTokenDto = plainToInstance(FetchTokenClassesDto, {tokenClasses: sale.value.selling.map((s:any) => s.tokenClassKey)});
      costTokens.value = await connect.fetchTokenClasses(saleTokenDto);
    } catch (error) {
      console.error(error);
    } finally {
      loading.value = false;
    }
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