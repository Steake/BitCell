//! Faucet web interface

use axum::response::{Html, IntoResponse};

/// Faucet page
pub async fn faucet_page() -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BitCell Testnet Faucet</title>
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
            min-height: 100vh;
            padding: 2rem;
        }

        .container {
            max-width: 800px;
            margin: 0 auto;
        }

        .header {
            text-align: center;
            margin-bottom: 2rem;
            padding: 2rem;
            background: rgba(0, 20, 15, 0.8);
            border: 2px solid #00ffaa;
            border-radius: 10px;
            box-shadow: 0 0 20px rgba(0, 255, 170, 0.3);
        }

        h1 {
            font-family: 'Orbitron', sans-serif;
            font-size: 2.5rem;
            text-shadow: 0 0 10px rgba(0, 255, 170, 0.8);
            margin-bottom: 0.5rem;
        }

        .subtitle {
            font-size: 1rem;
            opacity: 0.8;
        }

        .faucet-form {
            background: rgba(0, 20, 15, 0.8);
            border: 2px solid #00ffaa;
            border-radius: 10px;
            padding: 2rem;
            margin-bottom: 2rem;
            box-shadow: 0 0 20px rgba(0, 255, 170, 0.3);
        }

        .form-group {
            margin-bottom: 1.5rem;
        }

        label {
            display: block;
            margin-bottom: 0.5rem;
            font-weight: bold;
            color: #00ffaa;
        }

        input[type="text"] {
            width: 100%;
            padding: 0.75rem;
            background: rgba(0, 0, 0, 0.5);
            border: 2px solid #00ffaa;
            border-radius: 5px;
            color: #00ffaa;
            font-family: 'Share Tech Mono', monospace;
            font-size: 1rem;
        }

        input[type="text"]:focus {
            outline: none;
            box-shadow: 0 0 10px rgba(0, 255, 170, 0.5);
        }

        .btn {
            background: #00ffaa;
            color: #000;
            border: none;
            padding: 0.75rem 2rem;
            font-size: 1rem;
            font-weight: bold;
            font-family: 'Orbitron', sans-serif;
            border-radius: 5px;
            cursor: pointer;
            transition: all 0.3s;
            text-transform: uppercase;
        }

        .btn:hover:not(:disabled) {
            background: #00cc88;
            box-shadow: 0 0 15px rgba(0, 255, 170, 0.6);
            transform: translateY(-2px);
        }

        .btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }

        .info-box {
            background: rgba(0, 20, 15, 0.6);
            border: 1px solid #00ffaa;
            border-radius: 5px;
            padding: 1rem;
            margin-bottom: 2rem;
        }

        .info-item {
            display: flex;
            justify-content: space-between;
            padding: 0.5rem 0;
            border-bottom: 1px solid rgba(0, 255, 170, 0.2);
        }

        .info-item:last-child {
            border-bottom: none;
        }

        .info-label {
            opacity: 0.7;
        }

        .info-value {
            font-weight: bold;
        }

        .message {
            padding: 1rem;
            margin-bottom: 1rem;
            border-radius: 5px;
            display: none;
        }

        .message.success {
            background: rgba(0, 255, 170, 0.2);
            border: 1px solid #00ffaa;
            color: #00ffaa;
        }

        .message.error {
            background: rgba(255, 0, 100, 0.2);
            border: 1px solid #ff0064;
            color: #ff0064;
        }

        .message.show {
            display: block;
        }

        .history {
            background: rgba(0, 20, 15, 0.8);
            border: 2px solid #00ffaa;
            border-radius: 10px;
            padding: 2rem;
            box-shadow: 0 0 20px rgba(0, 255, 170, 0.3);
        }

        .history h2 {
            font-family: 'Orbitron', sans-serif;
            margin-bottom: 1rem;
            text-shadow: 0 0 5px rgba(0, 255, 170, 0.5);
        }

        .history-item {
            background: rgba(0, 0, 0, 0.3);
            border: 1px solid rgba(0, 255, 170, 0.3);
            border-radius: 5px;
            padding: 1rem;
            margin-bottom: 0.5rem;
            font-size: 0.9rem;
        }

        .history-address {
            font-weight: bold;
            word-break: break-all;
        }

        .history-time {
            opacity: 0.7;
            font-size: 0.8rem;
        }

        .loading {
            display: none;
            text-align: center;
            padding: 1rem;
        }

        .loading.show {
            display: block;
        }

        .spinner {
            border: 3px solid rgba(0, 255, 170, 0.3);
            border-top: 3px solid #00ffaa;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 0 auto;
        }

        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }

        .back-link {
            display: inline-block;
            margin-bottom: 1rem;
            color: #00ffaa;
            text-decoration: none;
            padding: 0.5rem 1rem;
            border: 1px solid #00ffaa;
            border-radius: 5px;
            transition: all 0.3s;
        }

        .back-link:hover {
            background: rgba(0, 255, 170, 0.1);
            box-shadow: 0 0 10px rgba(0, 255, 170, 0.3);
        }
    </style>
</head>
<body>
    <div class="container">
        <a href="/dashboard" class="back-link">← Back to Dashboard</a>

        <div class="header">
            <h1>🌊 BitCell Testnet Faucet</h1>
            <p class="subtitle">Get free testnet tokens for development and testing</p>
        </div>

        <div class="info-box" id="faucetInfo">
            <div class="info-item">
                <span class="info-label">Amount per request:</span>
                <span class="info-value" id="amountPerRequest">Loading...</span>
            </div>
            <div class="info-item">
                <span class="info-label">Rate limit:</span>
                <span class="info-value" id="rateLimit">Loading...</span>
            </div>
            <div class="info-item">
                <span class="info-label">Faucet balance:</span>
                <span class="info-value" id="faucetBalance">Loading...</span>
            </div>
            <div class="info-item">
                <span class="info-label">Requests today:</span>
                <span class="info-value" id="requestsToday">Loading...</span>
            </div>
        </div>

        <div class="faucet-form">
            <div id="message" class="message"></div>

            <div class="loading" id="loading">
                <div class="spinner"></div>
                <p>Processing your request...</p>
            </div>

            <form id="faucetForm">
                <div class="form-group">
                    <label for="address">Your BitCell Address</label>
                    <input 
                        type="text" 
                        id="address" 
                        placeholder="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
                        required
                    />
                </div>

                <button type="submit" class="btn" id="submitBtn">Request Tokens</button>
            </form>
        </div>

        <div class="history">
            <h2>Recent Distributions</h2>
            <div id="historyList">
                <p>Loading...</p>
            </div>
        </div>
    </div>

    <script>
        const API_BASE = window.location.origin;

        // Load faucet info
        async function loadFaucetInfo() {
            try {
                const response = await fetch(`${API_BASE}/api/faucet/info`);
                const data = await response.json();

                document.getElementById('amountPerRequest').textContent = 
                    `${(data.amount_per_request / 1e9).toFixed(2)} CELL`;
                document.getElementById('rateLimit').textContent = 
                    `${data.rate_limit_seconds / 3600} hour(s)`;
                document.getElementById('faucetBalance').textContent = 
                    `${(data.balance / 1e9).toFixed(2)} CELL`;

                // Load stats
                const statsResponse = await fetch(`${API_BASE}/api/faucet/stats`);
                const stats = await statsResponse.json();
                document.getElementById('requestsToday').textContent = stats.requests_last_day;
            } catch (error) {
                console.error('Failed to load faucet info:', error);
            }
        }

        // Load history
        async function loadHistory() {
            try {
                const response = await fetch(`${API_BASE}/api/faucet/history`);
                const history = await response.json();

                const historyList = document.getElementById('historyList');
                if (history.length === 0) {
                    historyList.innerHTML = '<p>No recent distributions</p>';
                    return;
                }

                historyList.innerHTML = history.slice(0, 10).map(item => {
                    const date = new Date(item.timestamp * 1000);
                    return `
                        <div class="history-item">
                            <div class="history-address">${item.address}</div>
                            <div>${(item.amount / 1e9).toFixed(2)} CELL</div>
                            <div class="history-time">${date.toLocaleString()}</div>
                        </div>
                    `;
                }).join('');
            } catch (error) {
                console.error('Failed to load history:', error);
            }
        }

        // Show message
        function showMessage(text, type) {
            const messageEl = document.getElementById('message');
            messageEl.textContent = text;
            messageEl.className = `message ${type} show`;
            setTimeout(() => {
                messageEl.classList.remove('show');
            }, 5000);
        }

        // Handle form submission
        document.getElementById('faucetForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const address = document.getElementById('address').value.trim();
            const submitBtn = document.getElementById('submitBtn');
            const loading = document.getElementById('loading');

            // Validate address
            if (!address.match(/^0x[0-9a-fA-F]{40}$/)) {
                showMessage('Invalid address format. Must be 0x followed by 40 hex characters.', 'error');
                return;
            }

            // Disable form
            submitBtn.disabled = true;
            loading.classList.add('show');

            try {
                const response = await fetch(`${API_BASE}/api/faucet/request`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ address }),
                });

                const data = await response.json();

                if (data.success) {
                    showMessage(
                        `Success! ${(data.amount / 1e9).toFixed(2)} CELL sent to ${address}. Transaction: ${data.tx_hash}`,
                        'success'
                    );
                    document.getElementById('address').value = '';
                    
                    // Reload info and history
                    setTimeout(() => {
                        loadFaucetInfo();
                        loadHistory();
                    }, 1000);
                } else {
                    showMessage(data.message || 'Request failed', 'error');
                }
            } catch (error) {
                showMessage(`Error: ${error.message}`, 'error');
            } finally {
                submitBtn.disabled = false;
                loading.classList.remove('show');
            }
        });

        // Initial load
        loadFaucetInfo();
        loadHistory();

        // Refresh every 30 seconds
        setInterval(() => {
            loadFaucetInfo();
            loadHistory();
        }, 30000);
    </script>
</body>
</html>
    "#;

    Html(html)
}
