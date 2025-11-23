//! Dashboard web interface

use axum::{
    response::{Html, IntoResponse},
    http::StatusCode,
};

/// Main dashboard page
pub async fn index() -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BitCell Admin Console</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: #0f172a;
            color: #e2e8f0;
            line-height: 1.6;
        }
        .header {
            background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
            padding: 1.5rem 2rem;
            box-shadow: 0 4px 6px rgba(0,0,0,0.3);
            border-bottom: 2px solid #3b82f6;
        }
        .header h1 {
            font-size: 2rem;
            font-weight: 700;
            background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        .header p { color: #94a3b8; margin-top: 0.5rem; }
        .container { max-width: 1400px; margin: 0 auto; padding: 2rem; }
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1.5rem;
            margin-top: 2rem;
        }
        .card {
            background: #1e293b;
            border-radius: 12px;
            padding: 1.5rem;
            box-shadow: 0 4px 6px rgba(0,0,0,0.2);
            border: 1px solid #334155;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        .card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 12px rgba(59, 130, 246, 0.2);
            border-color: #3b82f6;
        }
        .card h2 {
            font-size: 1.25rem;
            margin-bottom: 1rem;
            color: #3b82f6;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }
        .metric {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0.75rem 0;
            border-bottom: 1px solid #334155;
        }
        .metric:last-child { border-bottom: none; }
        .metric-label { color: #94a3b8; font-size: 0.9rem; }
        .metric-value {
            font-size: 1.5rem;
            font-weight: 700;
            color: #3b82f6;
        }
        .status {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.875rem;
            font-weight: 600;
        }
        .status-running {
            background: rgba(34, 197, 94, 0.1);
            color: #22c55e;
            border: 1px solid #22c55e;
        }
        .status-stopped {
            background: rgba(239, 68, 68, 0.1);
            color: #ef4444;
            border: 1px solid #ef4444;
        }
        .btn {
            background: #3b82f6;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 6px;
            cursor: pointer;
            font-size: 0.875rem;
            font-weight: 600;
            transition: background 0.2s;
        }
        .btn:hover { background: #2563eb; }
        .btn-danger {
            background: #ef4444;
        }
        .btn-danger:hover { background: #dc2626; }
        .node-list {
            margin-top: 1rem;
        }
        .node-item {
            background: #0f172a;
            padding: 1rem;
            border-radius: 8px;
            margin-bottom: 0.75rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .node-info h3 {
            font-size: 1rem;
            margin-bottom: 0.25rem;
        }
        .node-info p {
            font-size: 0.875rem;
            color: #64748b;
        }
        .actions {
            display: flex;
            gap: 0.5rem;
        }
        .loading {
            text-align: center;
            padding: 2rem;
            color: #64748b;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔬 BitCell Admin Console</h1>
        <p>Blockchain Management & Monitoring Dashboard</p>
    </div>

    <div class="container">
        <div class="grid">
            <!-- Chain Metrics -->
            <div class="card">
                <h2>⛓️ Chain Metrics</h2>
                <div class="metric">
                    <span class="metric-label">Block Height</span>
                    <span class="metric-value" id="chain-height">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Transactions</span>
                    <span class="metric-value" id="chain-txs">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Pending TX</span>
                    <span class="metric-value" id="chain-pending">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Avg Block Time</span>
                    <span class="metric-value" id="chain-blocktime">-</span>
                </div>
            </div>

            <!-- Network Metrics -->
            <div class="card">
                <h2>🌐 Network Metrics</h2>
                <div class="metric">
                    <span class="metric-label">Connected Peers</span>
                    <span class="metric-value" id="net-peers">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Bytes Sent</span>
                    <span class="metric-value" id="net-sent">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Bytes Received</span>
                    <span class="metric-value" id="net-recv">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Messages</span>
                    <span class="metric-value" id="net-msgs">-</span>
                </div>
            </div>

            <!-- EBSL Metrics -->
            <div class="card">
                <h2>🛡️ EBSL Metrics</h2>
                <div class="metric">
                    <span class="metric-label">Active Miners</span>
                    <span class="metric-value" id="ebsl-miners">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Banned Miners</span>
                    <span class="metric-value" id="ebsl-banned">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Avg Trust Score</span>
                    <span class="metric-value" id="ebsl-trust">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Slash Events</span>
                    <span class="metric-value" id="ebsl-slashes">-</span>
                </div>
            </div>

            <!-- System Metrics -->
            <div class="card">
                <h2>💻 System Metrics</h2>
                <div class="metric">
                    <span class="metric-label">Uptime</span>
                    <span class="metric-value" id="sys-uptime">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">CPU Usage</span>
                    <span class="metric-value" id="sys-cpu">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Memory</span>
                    <span class="metric-value" id="sys-mem">-</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Disk</span>
                    <span class="metric-value" id="sys-disk">-</span>
                </div>
            </div>
        </div>

        <!-- Nodes Section -->
        <div class="card" style="margin-top: 2rem;">
            <h2>🖥️ Registered Nodes</h2>
            <div id="nodes-list" class="node-list">
                <div class="loading">Loading nodes...</div>
            </div>
        </div>
    </div>

    <script>
        // Fetch and update metrics
        async function updateMetrics() {
            try {
                const response = await fetch('/api/metrics');
                const data = await response.json();

                // Chain metrics
                document.getElementById('chain-height').textContent = data.chain.height.toLocaleString();
                document.getElementById('chain-txs').textContent = data.chain.total_transactions.toLocaleString();
                document.getElementById('chain-pending').textContent = data.chain.pending_transactions;
                document.getElementById('chain-blocktime').textContent = data.chain.average_block_time.toFixed(1) + 's';

                // Network metrics
                document.getElementById('net-peers').textContent = `${data.network.connected_peers}/${data.network.total_peers}`;
                document.getElementById('net-sent').textContent = formatBytes(data.network.bytes_sent);
                document.getElementById('net-recv').textContent = formatBytes(data.network.bytes_received);
                document.getElementById('net-msgs').textContent = (data.network.messages_sent + data.network.messages_received).toLocaleString();

                // EBSL metrics
                document.getElementById('ebsl-miners').textContent = data.ebsl.active_miners;
                document.getElementById('ebsl-banned').textContent = data.ebsl.banned_miners;
                document.getElementById('ebsl-trust').textContent = (data.ebsl.average_trust_score * 100).toFixed(0) + '%';
                document.getElementById('ebsl-slashes').textContent = data.ebsl.total_slashing_events;

                // System metrics
                document.getElementById('sys-uptime').textContent = formatUptime(data.system.uptime_seconds);
                document.getElementById('sys-cpu').textContent = data.system.cpu_usage.toFixed(1) + '%';
                document.getElementById('sys-mem').textContent = formatBytes(data.system.memory_usage_mb * 1024 * 1024);
                document.getElementById('sys-disk').textContent = formatBytes(data.system.disk_usage_mb * 1024 * 1024);
            } catch (error) {
                console.error('Failed to fetch metrics:', error);
            }
        }

        // Fetch and display nodes
        async function updateNodes() {
            try {
                const response = await fetch('/api/nodes');
                const data = await response.json();

                const nodesList = document.getElementById('nodes-list');
                if (data.nodes.length === 0) {
                    nodesList.innerHTML = '<div class="loading">No nodes registered</div>';
                    return;
                }

                nodesList.innerHTML = data.nodes.map(node => `
                    <div class="node-item">
                        <div class="node-info">
                            <h3>${node.id}</h3>
                            <p>Type: ${node.node_type} | ${node.address}:${node.port}</p>
                        </div>
                        <div class="actions">
                            <span class="status status-${node.status}">${node.status}</span>
                            <button class="btn" onclick="startNode('${node.id}')" ${node.status === 'running' ? 'disabled' : ''}>Start</button>
                            <button class="btn btn-danger" onclick="stopNode('${node.id}')" ${node.status === 'stopped' ? 'disabled' : ''}>Stop</button>
                        </div>
                    </div>
                `).join('');
            } catch (error) {
                console.error('Failed to fetch nodes:', error);
            }
        }

        async function startNode(id) {
            try {
                await fetch(`/api/nodes/${id}/start`, { method: 'POST' });
                updateNodes();
            } catch (error) {
                console.error('Failed to start node:', error);
            }
        }

        async function stopNode(id) {
            try {
                await fetch(`/api/nodes/${id}/stop`, { method: 'POST' });
                updateNodes();
            } catch (error) {
                console.error('Failed to stop node:', error);
            }
        }

        function formatBytes(bytes) {
            if (bytes < 1024) return bytes + 'B';
            if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + 'KB';
            if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + 'MB';
            return (bytes / (1024 * 1024 * 1024)).toFixed(1) + 'GB';
        }

        function formatUptime(seconds) {
            const days = Math.floor(seconds / 86400);
            const hours = Math.floor((seconds % 86400) / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            if (days > 0) return `${days}d ${hours}h`;
            if (hours > 0) return `${hours}h ${minutes}m`;
            return `${minutes}m`;
        }

        // Initial load and auto-refresh
        updateMetrics();
        updateNodes();
        setInterval(updateMetrics, 5000);
        setInterval(updateNodes, 10000);
    </script>
</body>
</html>
    "#;

    (StatusCode::OK, Html(html))
}
