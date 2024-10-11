<template>
    <header id="top-header">
        <nav id="top-nav">
            <Logo/>
            <WalletConnect />
        </nav>
    </header>
</template>

<script setup>
import Logo from '../icons/Logo.vue';
import WalletConnect from './WalletConnect.vue';
</script>

<style scoped>
#top-header{
    width:100%;
    height:auto;
    padding:1rem 1.5rem;
    position:sticky;
    top:0;
    background:#000;
}
#top-nav{
    width:100%;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
}
</style>