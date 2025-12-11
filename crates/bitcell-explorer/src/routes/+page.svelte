<script>
import { onMount } from 'svelte';
import { getBlockNumber, getBlockByNumber } from '$lib/rpc.js';
import { shortHash, formatDate } from '$lib/utils.js';

let blocks = [];
let loading = true;
let error = null;
let currentBlock = 0;

async function loadBlocks() {
try {
loading = true;
error = null;

// Get current block number
const blockNumHex = await getBlockNumber();
currentBlock = parseInt(blockNumHex, 16);

// Load last 10 blocks
const blockPromises = [];
const start = Math.max(0, currentBlock - 9);

for (let i = currentBlock; i >= start; i--) {
blockPromises.push(getBlockByNumber(`0x${i.toString(16)}`, false));
}

blocks = await Promise.all(blockPromises);
loading = false;
} catch (e) {
error = e.message;
loading = false;
}
}

onMount(() => {
loadBlocks();
// Refresh every 10 seconds
const interval = setInterval(loadBlocks, 10000);
return () => clearInterval(interval);
});

function formatBlockNumber(hex) {
return parseInt(hex, 16);
}
</script>

<svelte:head>
<title>BitCell Block Explorer</title>
</svelte:head>

<div class="card">
<div style="display: flex; justify-content: space-between; align-items: center;">
<h2>⛓️ Recent Blocks</h2>
<button class="btn" on:click={loadBlocks} disabled={loading}>
🔄 Refresh
</button>
</div>

{#if loading && blocks.length === 0}
<div class="loading">Loading blocks...</div>
{:else if error}
<div class="error">Error: {error}</div>
{:else if blocks.length === 0}
<p>No blocks found</p>
{:else}
{#each blocks as block}
<a href="/block/{formatBlockNumber(block.number)}" class="item" role="button" tabindex="0">
<div class="item-header">
<span class="item-title">Block #{formatBlockNumber(block.number)}</span>
<span class="badge">{block.transactions?.length || 0} TX</span>
</div>
<div style="font-size: 0.85rem; color: rgba(0, 255, 170, 0.6);">
<div class="hash">Hash: {shortHash(block.hash)}</div>
<div>Proposer: {shortHash(block.miner)}</div>
<div>Time: {formatDate(parseInt(block.timestamp, 16))}</div>
</div>
</a>
{/each}
{/if}
</div>

<style>
a.item {
text-decoration: none;
color: inherit;
display: block;
}
</style>
