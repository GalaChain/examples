<template>
  <div class="info-container">
    <h1>About GalaChain Burn dApp</h1>
    
    <section>
      <h2>Purpose</h2>
      <p>This dApp serves as a template and reference implementation for building applications on GalaChain. It demonstrates core functionality including:</p>
      <ul>
        <li>Wallet connection and authentication</li>
        <li>User registration flow</li>
        <li>Balance checking</li>
        <li>Transaction signing and execution</li>
      </ul>
    </section>

    <section>
      <h2>Usage Tips</h2>
      <ul>
        <li>Ensure MetaMask is installed</li>
        <li>First-time users will need to complete registration</li>
        <li>Network fees of 1 GALA apply to transactions</li>
        <li>Always verify your balance before attempting burns</li>
      </ul>
    </section>

    <section>
      <h2>Extension Ideas</h2>
      <p>Here are some ways you could extend this template:</p>
      <ul>
        <li>Add transaction history tracking</li>
        <li>Create a token transfer interface</li>
        <li>Add fun visual effects to the burns</li>
        <li>Track user burns and build a leaderboard</li>
        <li>Build a game around burning tokens</li>
      </ul>
    </section>

    <section>
      <h2>Deployment Guide</h2>
      <ol>
        <li>Set up environment variables in .env file</li>
        <li>Build the project using <code>npm run build</code></li>
        <li>Deploy to static hosting (GitHub Pages, Netlify, Vercel)</li>
        <li>Ensure CORS and API endpoints are properly configured</li>
      </ol>
    </section>

    <section>
      <h2>Resources</h2>
      <ul>
        <li><a href="https://docs.galachain.com" target="_blank">GalaChain Documentation</a></li>
        <li><a href="https://github.com/GalaChain/sdk" target="_blank">GalaChain SDK</a></li>
        <li><a href="https://discord.gg/galachain" target="_blank">Developer Discord</a></li>
      </ul>
    </section>

    <button class="back-button" @click="$emit('back')">Back to Main Page</button>
  </div>
</template>

<script setup lang="ts">
defineEmits(['back'])
</script>

<style scoped>
.info-container {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

section {
  margin: 30px 0;
}

h1 {
  color: var(--primary-color);
  margin-bottom: 30px;
}

h2 {
  color: var(--primary-color);
  margin-bottom: 15px;
}

ul, ol {
  margin-left: 20px;
  line-height: 1.6;
}

li {
  margin: 8px 0;
}

a {
  color: var(--primary-color);
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

.back-button {
  margin-top: 30px;
  padding: 10px 20px;
  background-color: var(--primary-color);
  color: var(--background-color);
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.back-button:hover {
  background-color: var(--button-hover-color);
}

code {
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}
</style>
