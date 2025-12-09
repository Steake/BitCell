//! Block Explorer web interface

use axum::{
    response::{Html, IntoResponse},
    http::StatusCode,
};

/// Block Explorer page
pub async fn index() -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BitCell Block Explorer</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&family=Orbitron:wght@400;700;900&display=swap');

        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            font-family: 'Share Tech Mono', 'Courier New', monospace;
            background: #000000;
            background-image:
                linear-gradient(rgba(0, 255, 170, 0.03) 1px, transparent 1px),
                linear-gradient(90deg, rgba(0, 255, 170, 0.03) 1px, transparent 1px);
            background-size: 20px 20px;
            color: #00ffaa;
            line-height: 1.6;
            position: relative;
            min-height: 100vh;
        }

        body::before {
            content: '';
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: repeating-linear-gradient(
                0deg,
                rgba(0, 255, 170, 0.05) 0px,
                transparent 1px,
                transparent 2px,
                rgba(0, 255, 170, 0.05) 3px
            );
            pointer-events: none;
            z-index: 1000;
            animation: scanlines 8s linear infinite;
        }

        @keyframes scanlines {
            0% { transform: translateY(0); }
            100% { transform: translateY(20px); }
        }

        .header {
            background: linear-gradient(135deg, rgba(0, 0, 0, 0.9) 0%, rgba(10, 25, 20, 0.95) 100%);
            padding: 1.5rem 2rem;
            box-shadow:
                0 0 20px rgba(0, 255, 170, 0.3),
                0 4px 6px rgba(0,0,0,0.5),
                inset 0 -1px 0 rgba(0, 255, 170, 0.5);
            border-bottom: 2px solid #00ffaa;
            position: sticky;
            top: 0;
            z-index: 100;
        }

        .header h1 {
            font-family: 'Orbitron', monospace;
            font-size: 2rem;
            font-weight: 900;
            text-transform: uppercase;
            letter-spacing: 4px;
            background: linear-gradient(135deg, #00ffaa 0%, #00ff00 50%, #00ffaa 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            text-shadow:
                0 0 20px rgba(0, 255, 170, 0.5),
                0 0 40px rgba(0, 255, 170, 0.3);
        }

        .header nav {
            margin-top: 1rem;
            display: flex;
            gap: 2rem;
        }

        .header nav a {
            color: #00ffaa;
            text-decoration: none;
            opacity: 0.7;
            transition: opacity 0.3s;
            font-size: 0.9rem;
            letter-spacing: 1px;
        }

        .header nav a:hover {
            opacity: 1;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 2rem;
            position: relative;
            z-index: 1;
        }

        .search-bar {
            background: rgba(0, 10, 8, 0.85);
            backdrop-filter: blur(10px);
            border-radius: 4px;
            padding: 1.5rem;
            box-shadow:
                0 0 20px rgba(0, 255, 170, 0.2),
                inset 0 0 20px rgba(0, 255, 170, 0.05);
            border: 1px solid rgba(0, 255, 170, 0.3);
            margin-bottom: 2rem;
        }

        .search-bar input {
            width: 100%;
            padding: 1rem;
            background: rgba(0, 0, 0, 0.5);
            border: 1px solid rgba(0, 255, 170, 0.3);
            color: #00ffaa;
            font-family: 'Share Tech Mono', monospace;
            font-size: 1rem;
            border-radius: 4px;
            outline: none;
            transition: all 0.3s ease;
        }

        .search-bar input:focus {
            border-color: #00ffaa;
            box-shadow: 0 0 10px rgba(0, 255, 170, 0.3);
        }

        .card {
            background: rgba(0, 10, 8, 0.85);
            backdrop-filter: blur(10px);
            border-radius: 4px;
            padding: 1.5rem;
            box-shadow:
                0 0 20px rgba(0, 255, 170, 0.2),
                inset 0 0 20px rgba(0, 255, 170, 0.05),
                0 4px 6px rgba(0,0,0,0.5);
            border: 1px solid rgba(0, 255, 170, 0.3);
            margin-bottom: 1.5rem;
        }

        .card h2 {
            font-family: 'Orbitron', monospace;
            font-size: 1.3rem;
            font-weight: 700;
            margin-bottom: 1rem;
            color: #00ffaa;
            text-transform: uppercase;
            letter-spacing: 2px;
            text-shadow: 0 0 10px rgba(0, 255, 170, 0.5);
        }

        .item {
            background: rgba(0, 0, 0, 0.5);
            padding: 1rem;
            border-radius: 2px;
            margin-bottom: 0.75rem;
            border: 1px solid rgba(0, 255, 170, 0.2);
            transition: all 0.3s ease;
            cursor: pointer;
        }

        .item:hover {
            background: rgba(0, 255, 170, 0.05);
            border-color: #00ffaa;
            box-shadow: 0 0 15px rgba(0, 255, 170, 0.2);
        }

        .item-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 0.5rem;
        }

        .item-title {
            font-family: 'Orbitron', monospace;
            font-size: 1rem;
            color: #00ffaa;
            text-shadow: 0 0 5px rgba(0, 255, 170, 0.3);
        }

        .item-meta {
            font-size: 0.85rem;
            color: rgba(0, 255, 170, 0.6);
        }

        .badge {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 2px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 1px;
            background: rgba(0, 255, 170, 0.1);
            color: #00ffaa;
            border: 1px solid #00ffaa;
            box-shadow: 0 0 10px rgba(0, 255, 170, 0.2);
        }

        .hash {
            font-family: 'Share Tech Mono', monospace;
            font-size: 0.85rem;
            color: rgba(0, 255, 170, 0.8);
            word-break: break-all;
        }

        .loading {
            text-align: center;
            padding: 3rem;
            color: rgba(0, 255, 170, 0.5);
            font-size: 1.1rem;
            text-transform: uppercase;
            letter-spacing: 2px;
            animation: pulse 2s ease-in-out infinite;
        }

        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.6; }
        }

        .btn {
            font-family: 'Share Tech Mono', monospace;
            background: rgba(0, 255, 170, 0.1);
            color: #00ffaa;
            border: 1px solid #00ffaa;
            padding: 0.5rem 1rem;
            border-radius: 2px;
            cursor: pointer;
            font-size: 0.8rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 1px;
            transition: all 0.3s ease;
            box-shadow: 0 0 10px rgba(0, 255, 170, 0.2);
        }

        .btn:hover {
            background: rgba(0, 255, 170, 0.2);
            box-shadow: 0 0 20px rgba(0, 255, 170, 0.4);
            transform: translateY(-2px);
        }

        .detail-modal {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.95);
            z-index: 2000;
            overflow-y: auto;
            padding: 2rem;
        }

        .detail-modal.active {
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .detail-container {
            max-width: 1200px;
            width: 100%;
            background: rgba(0, 10, 8, 0.95);
            border: 2px solid #00ffaa;
            border-radius: 8px;
            padding: 2rem;
            box-shadow: 0 0 40px rgba(0, 255, 170, 0.5);
            max-height: 90vh;
            overflow-y: auto;
        }

        .detail-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 2rem;
        }

        .detail-header h2 {
            font-family: 'Orbitron', monospace;
            font-size: 1.5rem;
            color: #00ffaa;
            text-shadow: 0 0 20px rgba(0, 255, 170, 0.5);
        }

        .close-btn {
            background: rgba(255, 0, 100, 0.1);
            color: #ff0064;
            border: 1px solid #ff0064;
            padding: 0.5rem 1rem;
            border-radius: 2px;
            cursor: pointer;
            font-family: 'Share Tech Mono', monospace;
            font-size: 0.9rem;
            transition: all 0.3s ease;
        }

        .close-btn:hover {
            background: rgba(255, 0, 100, 0.2);
            box-shadow: 0 0 20px rgba(255, 0, 100, 0.4);
        }

        .detail-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }

        .detail-field {
            background: rgba(0, 0, 0, 0.5);
            padding: 1rem;
            border-radius: 4px;
            border: 1px solid rgba(0, 255, 170, 0.2);
        }

        .detail-label {
            color: rgba(0, 255, 170, 0.6);
            font-size: 0.8rem;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 0.5rem;
        }

        .detail-value {
            color: #00ffaa;
            font-size: 1rem;
            word-break: break-all;
        }

        .trust-meter {
            width: 100%;
            height: 30px;
            background: rgba(0, 0, 0, 0.5);
            border-radius: 15px;
            overflow: hidden;
            border: 1px solid rgba(0, 255, 170, 0.3);
            margin-top: 0.5rem;
        }

        .trust-fill {
            height: 100%;
            background: linear-gradient(90deg, #ff0064, #ffaa00, #00ff00);
            transition: width 0.5s ease;
            box-shadow: 0 0 10px rgba(0, 255, 170, 0.5);
        }

        .tabs {
            display: flex;
            gap: 1rem;
            margin-bottom: 1rem;
            border-bottom: 1px solid rgba(0, 255, 170, 0.3);
        }

        .tab {
            padding: 0.75rem 1.5rem;
            background: transparent;
            border: none;
            color: rgba(0, 255, 170, 0.6);
            font-family: 'Share Tech Mono', monospace;
            font-size: 0.9rem;
            cursor: pointer;
            text-transform: uppercase;
            letter-spacing: 1px;
            border-bottom: 2px solid transparent;
            transition: all 0.3s ease;
        }

        .tab:hover {
            color: #00ffaa;
        }

        .tab.active {
            color: #00ffaa;
            border-bottom-color: #00ffaa;
        }

        .tab-content {
            display: none;
        }

        .tab-content.active {
            display: block;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔍 BitCell Block Explorer</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/explorer" class="active">Explorer</a>
        </nav>
    </div>

    <div class="container">
        <!-- Search Bar -->
        <div class="search-bar">
            <input 
                type="text" 
                id="search-input" 
                placeholder="Search by block height, hash, transaction hash, or address..."
                onkeypress="if(event.key === 'Enter') performSearch()"
            >
        </div>

        <!-- Tabs -->
        <div class="tabs">
            <button class="tab active" onclick="switchTab('blocks')">Recent Blocks</button>
            <button class="tab" onclick="switchTab('transactions')">Recent Transactions</button>
        </div>

        <!-- Blocks Tab -->
        <div id="blocks-tab" class="tab-content active">
            <div class="card">
                <h2>⛓️ Recent Blocks</h2>
                <div id="blocks-list">
                    <div class="loading">Loading blocks...</div>
                </div>
            </div>
        </div>

        <!-- Transactions Tab -->
        <div id="transactions-tab" class="tab-content">
            <div class="card">
                <h2>💸 Recent Transactions</h2>
                <div id="transactions-list">
                    <div class="loading">Loading transactions...</div>
                </div>
            </div>
        </div>
    </div>

    <!-- Block Detail Modal -->
    <div id="block-modal" class="detail-modal">
        <div class="detail-container">
            <div class="detail-header">
                <h2>Block #<span id="block-height"></span></h2>
                <button class="close-btn" onclick="closeBlockModal()">✕ Close</button>
            </div>
            <div id="block-detail-content"></div>
        </div>
    </div>

    <!-- Transaction Detail Modal -->
    <div id="tx-modal" class="detail-modal">
        <div class="detail-container">
            <div class="detail-header">
                <h2>Transaction Details</h2>
                <button class="close-btn" onclick="closeTxModal()">✕ Close</button>
            </div>
            <div id="tx-detail-content"></div>
        </div>
    </div>

    <!-- Account Detail Modal -->
    <div id="account-modal" class="detail-modal">
        <div class="detail-container">
            <div class="detail-header">
                <h2>Account Details</h2>
                <button class="close-btn" onclick="closeAccountModal()">✕ Close</button>
            </div>
            <div id="account-detail-content"></div>
        </div>
    </div>

    <script>
        // Tab switching
        function switchTab(tab) {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            
            event.target.classList.add('active');
            document.getElementById(tab + '-tab').classList.add('active');
        }

        // Load blocks
        async function loadBlocks() {
            const blocksList = document.getElementById('blocks-list');
            
            try {
                const response = await fetch('/api/blocks');
                if (!response.ok) throw new Error('Failed to fetch blocks');
                
                const data = await response.json();
                
                if (data.blocks.length === 0) {
                    blocksList.innerHTML = '<div class="loading">No blocks found</div>';
                    return;
                }
                
                blocksList.innerHTML = data.blocks.map(block => `
                    <div class="item" onclick="showBlockDetail(${block.height})">
                        <div class="item-header">
                            <span class="item-title">Block #${block.height}</span>
                            <span class="badge">${block.battle_count} Battle${block.battle_count !== 1 ? 's' : ''}</span>
                        </div>
                        <div class="item-meta">
                            <div class="hash">Hash: ${block.hash}</div>
                            <div>Proposer: ${block.proposer} • ${new Date(block.timestamp * 1000).toLocaleString()}</div>
                            <div>Transactions: ${block.transaction_count}</div>
                        </div>
                    </div>
                `).join('');
            } catch (error) {
                console.error('Failed to load blocks:', error);
                blocksList.innerHTML = '<div class="loading">Failed to load blocks</div>';
            }
        }

        // Load transactions (mock for now)
        async function loadTransactions() {
            const txList = document.getElementById('transactions-list');
            txList.innerHTML = '<div class="loading">Transaction history coming soon...</div>';
        }

        // Search functionality
        async function performSearch() {
            const query = document.getElementById('search-input').value.trim();
            if (!query) return;
            
            try {
                const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
                if (!response.ok) {
                    alert('No results found');
                    return;
                }
                
                const data = await response.json();
                
                if (data.results.length === 0) {
                    alert('No results found');
                    return;
                }
                
                const result = data.results[0];
                
                // Handle tagged enum serialization with serde
                if (result.type === 'Block') {
                    showBlockDetail(result.data.height);
                } else if (result.type === 'Transaction') {
                    showTransactionDetail(result.data.hash);
                } else if (result.type === 'Account') {
                    showAccountDetail(result.data.address);
                }
            } catch (error) {
                console.error('Search failed:', error);
                alert('Search failed: ' + error.message);
            }
        }

        // Show block detail
        async function showBlockDetail(height) {
            const modal = document.getElementById('block-modal');
            const content = document.getElementById('block-detail-content');
            
            document.getElementById('block-height').textContent = height;
            content.innerHTML = '<div class="loading">Loading block details...</div>';
            modal.classList.add('active');
            
            try {
                const response = await fetch(`/api/blocks/${height}`);
                if (!response.ok) throw new Error('Failed to fetch block');
                
                const block = await response.json();
                
                content.innerHTML = `
                    <div class="detail-grid">
                        <div class="detail-field">
                            <div class="detail-label">Block Height</div>
                            <div class="detail-value">${block.height}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Block Hash</div>
                            <div class="detail-value hash">${block.hash}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Previous Hash</div>
                            <div class="detail-value hash">${block.prev_hash}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Proposer</div>
                            <div class="detail-value">${block.proposer}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Timestamp</div>
                            <div class="detail-value">${new Date(block.timestamp * 1000).toLocaleString()}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Transactions</div>
                            <div class="detail-value">${block.transactions.length}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">State Root</div>
                            <div class="detail-value hash">${block.state_root}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">TX Root</div>
                            <div class="detail-value hash">${block.tx_root}</div>
                        </div>
                    </div>
                    <button class="btn" onclick="window.location.href='/?block=${height}'">View Battle Replay</button>
                `;
            } catch (error) {
                console.error('Failed to load block details:', error);
                content.innerHTML = '<div class="loading">Failed to load block details</div>';
            }
        }

        // Show transaction detail
        async function showTransactionDetail(hash) {
            const modal = document.getElementById('tx-modal');
            const content = document.getElementById('tx-detail-content');
            
            content.innerHTML = '<div class="loading">Loading transaction details...</div>';
            modal.classList.add('active');
            
            try {
                const response = await fetch(`/api/transactions/${hash}`);
                if (!response.ok) throw new Error('Failed to fetch transaction');
                
                const tx = await response.json();
                
                content.innerHTML = `
                    <div class="detail-grid">
                        <div class="detail-field">
                            <div class="detail-label">Transaction Hash</div>
                            <div class="detail-value hash">${tx.hash}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Block Height</div>
                            <div class="detail-value">#${tx.block_height}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">From</div>
                            <div class="detail-value hash" style="cursor: pointer;" onclick="showAccountDetail('${tx.from}')">${tx.from}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">To</div>
                            <div class="detail-value hash" style="cursor: pointer;" onclick="showAccountDetail('${tx.to}')">${tx.to}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Amount</div>
                            <div class="detail-value">${(tx.amount / 1000000).toFixed(6)} CELL</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Fee</div>
                            <div class="detail-value">${tx.fee} units</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Nonce</div>
                            <div class="detail-value">${tx.nonce}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Status</div>
                            <div class="detail-value"><span class="badge">${tx.status}</span></div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Timestamp</div>
                            <div class="detail-value">${new Date(tx.timestamp * 1000).toLocaleString()}</div>
                        </div>
                    </div>
                `;
            } catch (error) {
                console.error('Failed to load transaction details:', error);
                content.innerHTML = '<div class="loading">Failed to load transaction details</div>';
            }
        }

        // Show account detail
        async function showAccountDetail(address) {
            const modal = document.getElementById('account-modal');
            const content = document.getElementById('account-detail-content');
            
            content.innerHTML = '<div class="loading">Loading account details...</div>';
            modal.classList.add('active');
            
            try {
                const [accountResponse, trustResponse, txResponse] = await Promise.all([
                    fetch(`/api/accounts/${address}`),
                    fetch(`/api/accounts/${address}/trust`),
                    fetch(`/api/accounts/${address}/transactions`)
                ]);
                
                if (!accountResponse.ok) throw new Error('Failed to fetch account');
                
                const account = await accountResponse.json();
                const trust = trustResponse.ok ? await trustResponse.json() : null;
                const txHistory = txResponse.ok ? await txResponse.json() : null;
                
                const trustPercent = (trust ? trust.trust_score * 100 : 0).toFixed(1);
                
                content.innerHTML = `
                    <div class="detail-grid">
                        <div class="detail-field">
                            <div class="detail-label">Address</div>
                            <div class="detail-value hash">${account.address}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Balance</div>
                            <div class="detail-value">${(account.balance / 1000000).toFixed(6)} CELL</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Nonce</div>
                            <div class="detail-value">${account.nonce}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">Transaction Count</div>
                            <div class="detail-value">${account.transaction_count}</div>
                        </div>
                    </div>
                    
                    ${trust ? `
                        <div class="card" style="margin-top: 2rem;">
                            <h2>🛡️ Trust Score</h2>
                            <div class="detail-grid">
                                <div class="detail-field">
                                    <div class="detail-label">Trust Score</div>
                                    <div class="detail-value">${trustPercent}%</div>
                                    <div class="trust-meter">
                                        <div class="trust-fill" style="width: ${trustPercent}%"></div>
                                    </div>
                                </div>
                                <div class="detail-field">
                                    <div class="detail-label">Status</div>
                                    <div class="detail-value"><span class="badge">${trust.status}</span></div>
                                </div>
                                <div class="detail-field">
                                    <div class="detail-label">Positive Evidence</div>
                                    <div class="detail-value">${trust.positive_evidence}</div>
                                </div>
                                <div class="detail-field">
                                    <div class="detail-label">Negative Evidence</div>
                                    <div class="detail-value">${trust.negative_evidence}</div>
                                </div>
                                <div class="detail-field">
                                    <div class="detail-label">Blocks Proposed</div>
                                    <div class="detail-value">${trust.total_blocks_proposed}</div>
                                </div>
                                <div class="detail-field">
                                    <div class="detail-label">Slashing Events</div>
                                    <div class="detail-value">${trust.slashing_events}</div>
                                </div>
                            </div>
                        </div>
                    ` : ''}
                    
                    ${txHistory && txHistory.transactions.length > 0 ? `
                        <div class="card" style="margin-top: 2rem;">
                            <h2>💸 Recent Transactions</h2>
                            ${txHistory.transactions.map(tx => `
                                <div class="item" onclick="showTransactionDetail('${tx.hash}')">
                                    <div class="item-header">
                                        <span class="item-title">${tx.from === address ? 'Sent' : 'Received'} ${(tx.amount / 1000000).toFixed(6)} CELL</span>
                                        <span class="badge">${tx.status}</span>
                                    </div>
                                    <div class="item-meta">
                                        <div class="hash">Hash: ${tx.hash}</div>
                                        <div>${tx.from === address ? 'To: ' + tx.to : 'From: ' + tx.from}</div>
                                        <div>Block: #${tx.block_height} • ${new Date(tx.timestamp * 1000).toLocaleString()}</div>
                                    </div>
                                </div>
                            `).join('')}
                        </div>
                    ` : ''}
                `;
            } catch (error) {
                console.error('Failed to load account details:', error);
                content.innerHTML = '<div class="loading">Failed to load account details</div>';
            }
        }

        // Modal close functions
        function closeBlockModal() {
            document.getElementById('block-modal').classList.remove('active');
        }

        function closeTxModal() {
            document.getElementById('tx-modal').classList.remove('active');
        }

        function closeAccountModal() {
            document.getElementById('account-modal').classList.remove('active');
        }

        // Initialize
        loadBlocks();
        setInterval(loadBlocks, 10000); // Refresh every 10 seconds
    </script>
</body>
</html>
    "#;

    (StatusCode::OK, Html(html))
}
