/**
 * BitCell RPC Client
 * Connects to BitCell node JSON-RPC endpoint
 */

const RPC_ENDPOINT = '/rpc';

let requestId = 1;

/**
 * Make a JSON-RPC call to the BitCell node
 * @param {string} method - RPC method name
 * @param {any[]} params - Method parameters
 * @returns {Promise<any>} - RPC result
 */
export async function rpcCall(method, params = []) {
const response = await fetch(RPC_ENDPOINT, {
method: 'POST',
headers: {
'Content-Type': 'application/json',
},
body: JSON.stringify({
jsonrpc: '2.0',
id: requestId++,
method,
params
})
});

if (!response.ok) {
throw new Error(`RPC request failed: ${response.statusText}`);
}

const data = await response.json();

if (data.error) {
throw new Error(data.error.message || 'RPC error');
}

return data.result;
}

/**
 * Get current block number
 */
export async function getBlockNumber() {
return await rpcCall('eth_blockNumber');
}

/**
 * Get block by number
 * @param {number|string} blockNumber - Block number or 'latest'
 * @param {boolean} fullTx - Include full transaction objects
 */
export async function getBlockByNumber(blockNumber, fullTx = true) {
return await rpcCall('eth_getBlockByNumber', [blockNumber, fullTx]);
}

/**
 * Get transaction by hash
 * @param {string} hash - Transaction hash
 */
export async function getTransactionByHash(hash) {
return await rpcCall('eth_getTransactionByHash', [hash]);
}

/**
 * Get account balance
 * @param {string} address - Account address
 */
export async function getBalance(address) {
return await rpcCall('eth_getBalance', [address, 'latest']);
}

/**
 * Get transaction count (nonce) for account
 * @param {string} address - Account address
 */
export async function getTransactionCount(address) {
return await rpcCall('eth_getTransactionCount', [address, 'latest']);
}

/**
 * Get node information
 */
export async function getNodeInfo() {
return await rpcCall('bitcell_getNodeInfo');
}

/**
 * Get tournament state
 */
export async function getTournamentState() {
return await rpcCall('bitcell_getTournamentState');
}

/**
 * Get battle replay data
 * @param {number} blockHeight - Block height
 */
export async function getBattleReplay(blockHeight) {
return await rpcCall('bitcell_getBattleReplay', [blockHeight]);
}

/**
 * Get miner stats
 * @param {string} address - Miner address
 */
export async function getMinerStats(address) {
return await rpcCall('bitcell_getMinerStats', [address]);
}

/**
 * Search for blocks, transactions, or addresses
 * @param {string} query - Search query
 */
export async function search(query) {
const trimmed = query.trim();

// Check if numeric (block height)
if (/^\d+$/.test(trimmed)) {
const blockNum = parseInt(trimmed, 10);
try {
const block = await getBlockByNumber(blockNum, false);
return {
type: 'block',
data: block
};
} catch (e) {
throw new Error('Block not found');
}
}

// Check if transaction hash (0x + 64 hex)
if (/^0x[0-9a-fA-F]{64}$/.test(trimmed)) {
try {
const tx = await getTransactionByHash(trimmed);
if (tx) {
return {
type: 'transaction',
data: tx
};
}
} catch (e) {
throw new Error('Transaction not found');
}
}

// Check if address (0x + 40 hex)
if (/^0x[0-9a-fA-F]{40}$/.test(trimmed)) {
try {
const balance = await getBalance(trimmed);
const nonce = await getTransactionCount(trimmed);
return {
type: 'account',
data: {
address: trimmed,
balance,
nonce
}
};
} catch (e) {
throw new Error('Account not found');
}
}

throw new Error('Invalid search query');
}
