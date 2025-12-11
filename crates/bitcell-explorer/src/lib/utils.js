/**
 * Utility functions for the block explorer
 */

/**
 * Format a hex value as a shortened hash
 * @param {string} hash - Full hash
 * @param {number} start - Characters to show at start
 * @param {number} end - Characters to show at end
 */
export function shortHash(hash, start = 6, end = 4) {
if (!hash || hash.length < start + end) return hash;
return `${hash.slice(0, start + 2)}...${hash.slice(-end)}`;
}

/**
 * Format timestamp to readable date
 * @param {number} timestamp - Unix timestamp
 */
export function formatDate(timestamp) {
return new Date(timestamp * 1000).toLocaleString();
}

/**
 * Format balance from wei to CELL
 * @param {string|number} wei - Balance in wei
 */
export function formatBalance(wei) {
const balance = typeof wei === 'string' ? parseInt(wei, 16) : wei;
return (balance / 1000000).toFixed(6);
}

/**
 * Escape HTML to prevent XSS
 * @param {string} text - Text to escape
 */
export function escapeHtml(text) {
const div = document.createElement('div');
div.textContent = text;
return div.innerHTML;
}

/**
 * Validate hex address format
 * @param {string} address - Address to validate
 */
export function isValidAddress(address) {
return /^0x[0-9a-fA-F]{40}$/.test(address);
}

/**
 * Validate transaction hash format
 * @param {string} hash - Hash to validate
 */
export function isValidTxHash(hash) {
return /^0x[0-9a-fA-F]{64}$/.test(hash);
}
