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
            position: relative;
            overflow: hidden;
        }

        .header::before {
            content: '';
            position: absolute;
            top: 0;
            left: -100%;
            width: 100%;
            height: 100%;
            background: linear-gradient(90deg, transparent, rgba(0, 255, 170, 0.1), transparent);
            animation: sweep 3s ease-in-out infinite;
        }

        @keyframes sweep {
            0%, 100% { left: -100%; }
            50% { left: 100%; }
        }

        .header h1 {
            font-family: 'Orbitron', monospace;
            font-size: 2.5rem;
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
            position: relative;
            z-index: 1;
        }

        .header p {
            color: #00ffaa;
            margin-top: 0.5rem;
            opacity: 0.7;
            letter-spacing: 2px;
            font-size: 0.9rem;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 2rem;
            position: relative;
            z-index: 1;
        }

        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1.5rem;
            margin-top: 2rem;
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
            transition: all 0.3s ease;
            position: relative;
            overflow: hidden;
        }

        .card::before {
            content: '';
            position: absolute;
            top: -2px;
            left: -2px;
            right: -2px;
            bottom: -2px;
            background: linear-gradient(45deg, #00ffaa, #00ff00, #00ffaa);
            opacity: 0;
            transition: opacity 0.3s ease;
            z-index: -1;
            border-radius: 4px;
        }

        .card:hover {
            transform: translateY(-4px);
            box-shadow:
                0 0 40px rgba(0, 255, 170, 0.4),
                inset 0 0 30px rgba(0, 255, 170, 0.1),
                0 8px 12px rgba(0,0,0,0.7);
            border-color: #00ffaa;
        }

        .card:hover::before {
            opacity: 0.2;
        }

        .card h2 {
            font-family: 'Orbitron', monospace;
            font-size: 1.1rem;
            font-weight: 700;
            margin-bottom: 1rem;
            color: #00ffaa;
            text-transform: uppercase;
            letter-spacing: 2px;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            text-shadow: 0 0 10px rgba(0, 255, 170, 0.5);
        }
        .metric {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0.75rem 0;
            border-bottom: 1px solid rgba(0, 255, 170, 0.1);
        }

        .metric:last-child { border-bottom: none; }

        .metric-label {
            color: rgba(0, 255, 170, 0.6);
            font-size: 0.85rem;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .metric-value {
            font-family: 'Orbitron', monospace;
            font-size: 1.5rem;
            font-weight: 700;
            color: #00ffaa;
            text-shadow: 0 0 10px rgba(0, 255, 170, 0.5);
        }

        .status {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 2px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .status-running {
            background: rgba(0, 255, 0, 0.1);
            color: #00ff00;
            border: 1px solid #00ff00;
            box-shadow: 0 0 10px rgba(0, 255, 0, 0.3);
            animation: pulse-green 2s ease-in-out infinite;
        }

        .status-stopped {
            background: rgba(255, 0, 100, 0.1);
            color: #ff0064;
            border: 1px solid #ff0064;
        }

        @keyframes pulse-green {
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

        .btn:disabled {
            opacity: 0.3;
            cursor: not-allowed;
        }

        .btn-danger {
            background: rgba(255, 0, 100, 0.1);
            color: #ff0064;
            border-color: #ff0064;
            box-shadow: 0 0 10px rgba(255, 0, 100, 0.2);
        }

        .btn-danger:hover {
            background: rgba(255, 0, 100, 0.2);
            box-shadow: 0 0 20px rgba(255, 0, 100, 0.4);
        }
        .node-list {
            margin-top: 1rem;
        }

        .node-item {
            background: rgba(0, 0, 0, 0.5);
            padding: 1rem;
            border-radius: 2px;
            margin-bottom: 0.75rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            border: 1px solid rgba(0, 255, 170, 0.2);
            transition: all 0.3s ease;
        }

        .node-item:hover {
            background: rgba(0, 255, 170, 0.05);
            border-color: #00ffaa;
            box-shadow: 0 0 15px rgba(0, 255, 170, 0.2);
        }

        .node-info h3 {
            font-family: 'Orbitron', monospace;
            font-size: 1rem;
            margin-bottom: 0.25rem;
            color: #00ffaa;
            text-shadow: 0 0 5px rgba(0, 255, 170, 0.3);
        }

        .node-info p {
            font-size: 0.8rem;
            color: rgba(0, 255, 170, 0.5);
        }

        .actions {
            display: flex;
            gap: 0.5rem;
        }

        .loading {
            text-align: center;
            padding: 2rem;
            color: rgba(0, 255, 170, 0.5);
            font-family: 'Share Tech Mono', monospace;
            text-transform: uppercase;
            letter-spacing: 2px;
            animation: pulse-green 2s ease-in-out infinite;
        }

        input[type="number"],
        input[type="range"],
        select {
            font-family: 'Share Tech Mono', monospace;
            background: rgba(0, 0, 0, 0.5);
            color: #00ffaa;
            border: 1px solid rgba(0, 255, 170, 0.3);
            outline: none;
        }

        input[type="number"]:focus,
        select:focus {
            border-color: #00ffaa;
            box-shadow: 0 0 10px rgba(0, 255, 170, 0.3);
        }

        input[type="range"] {
            accent-color: #00ffaa;
        }

        canvas {
            image-rendering: pixelated;
        }

        /* Setup Wizard Styles */
        .wizard-overlay {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.95);
            z-index: 2000;
            overflow-y: auto;
        }

        .wizard-overlay.active {
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 2rem;
        }

        .wizard-container {
            max-width: 800px;
            width: 100%;
            background: rgba(0, 10, 8, 0.95);
            border: 2px solid #00ffaa;
            border-radius: 8px;
            padding: 2rem;
            box-shadow: 0 0 40px rgba(0, 255, 170, 0.5);
        }

        .wizard-header {
            text-align: center;
            margin-bottom: 2rem;
        }

        .wizard-header h2 {
            font-family: 'Orbitron', monospace;
            font-size: 2rem;
            color: #00ffaa;
            text-shadow: 0 0 20px rgba(0, 255, 170, 0.5);
            margin-bottom: 0.5rem;
        }

        .wizard-header p {
            color: rgba(0, 255, 170, 0.7);
            font-size: 0.9rem;
        }

        .wizard-step {
            display: none;
        }

        .wizard-step.active {
            display: block;
        }

        .wizard-progress {
            display: flex;
            justify-content: space-between;
            margin-bottom: 2rem;
            position: relative;
        }

        .wizard-progress::before {
            content: '';
            position: absolute;
            top: 50%;
            left: 0;
            right: 0;
            height: 2px;
            background: rgba(0, 255, 170, 0.2);
            z-index: -1;
        }

        .progress-step {
            flex: 1;
            text-align: center;
            position: relative;
        }

        .progress-dot {
            width: 40px;
            height: 40px;
            border-radius: 50%;
            background: rgba(0, 0, 0, 0.8);
            border: 2px solid rgba(0, 255, 170, 0.3);
            margin: 0 auto 0.5rem;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: bold;
            transition: all 0.3s ease;
        }

        .progress-step.completed .progress-dot {
            background: rgba(0, 255, 170, 0.2);
            border-color: #00ffaa;
            box-shadow: 0 0 15px rgba(0, 255, 170, 0.5);
        }

        .progress-step.active .progress-dot {
            background: #00ffaa;
            color: #000;
            border-color: #00ffaa;
            box-shadow: 0 0 20px rgba(0, 255, 170, 0.7);
            animation: pulse-dot 2s ease-in-out infinite;
        }

        @keyframes pulse-dot {
            0%, 100% { transform: scale(1); }
            50% { transform: scale(1.1); }
        }

        .progress-label {
            font-size: 0.75rem;
            color: rgba(0, 255, 170, 0.5);
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .progress-step.active .progress-label,
        .progress-step.completed .progress-label {
            color: #00ffaa;
        }

        .form-group {
            margin-bottom: 1.5rem;
        }

        .form-group label {
            display: block;
            margin-bottom: 0.5rem;
            color: #00ffaa;
            font-size: 0.9rem;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .form-group input,
        .form-group select {
            width: 100%;
            padding: 0.75rem;
            background: rgba(0, 0, 0, 0.5);
            border: 1px solid rgba(0, 255, 170, 0.3);
            color: #00ffaa;
            font-family: 'Share Tech Mono', monospace;
            border-radius: 4px;
            outline: none;
            transition: all 0.3s ease;
        }

        .form-group input:focus,
        .form-group select:focus {
            border-color: #00ffaa;
            box-shadow: 0 0 10px rgba(0, 255, 170, 0.3);
        }

        .form-group small {
            display: block;
            margin-top: 0.25rem;
            color: rgba(0, 255, 170, 0.5);
            font-size: 0.8rem;
        }

        .node-list-wizard {
            margin: 1rem 0;
            max-height: 200px;
            overflow-y: auto;
        }

        .node-item-wizard {
            background: rgba(0, 0, 0, 0.5);
            border: 1px solid rgba(0, 255, 170, 0.2);
            border-radius: 4px;
            padding: 0.75rem;
            margin-bottom: 0.5rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .node-item-wizard .node-id {
            font-weight: bold;
            color: #00ffaa;
        }

        .node-item-wizard .node-type {
            font-size: 0.8rem;
            color: rgba(0, 255, 170, 0.6);
            text-transform: uppercase;
        }

        .wizard-actions {
            display: flex;
            gap: 1rem;
            margin-top: 2rem;
            justify-content: space-between;
        }

        .wizard-actions button {
            flex: 1;
        }

        .btn-secondary {
            background: rgba(0, 255, 170, 0.05);
            border: 1px solid rgba(0, 255, 170, 0.3);
        }

        .btn-secondary:hover {
            background: rgba(0, 255, 170, 0.1);
        }

        .success-message {
            text-align: center;
            padding: 2rem;
        }

        .success-icon {
            font-size: 4rem;
            margin-bottom: 1rem;
            animation: bounce 1s ease infinite;
        }

        @keyframes bounce {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-10px); }
        }

        .success-message h3 {
            font-family: 'Orbitron', monospace;
            color: #00ffaa;
            margin-bottom: 1rem;
            font-size: 1.5rem;
        }
    </style>
</head>
<body>
    <!-- Setup Wizard Overlay -->
    <div id="wizard-overlay" class="wizard-overlay">
        <div class="wizard-container">
            <div class="wizard-header">
                <h2>⚙️ BitCell Setup Wizard</h2>
                <p>Configure your administrative console</p>
            </div>

            <!-- Progress Indicator -->
            <div class="wizard-progress">
                <div class="progress-step active" data-step="1">
                    <div class="progress-dot">1</div>
                    <div class="progress-label">Paths</div>
                </div>
                <div class="progress-step" data-step="2">
                    <div class="progress-dot">2</div>
                    <div class="progress-label">Nodes</div>
                </div>
                <div class="progress-step" data-step="3">
                    <div class="progress-dot">3</div>
                    <div class="progress-label">Complete</div>
                </div>
            </div>

            <!-- Step 1: Paths Configuration -->
            <div id="wizard-step-1" class="wizard-step active">
                <h3 style="color: #00ffaa; margin-bottom: 1rem;">📁 Configure Paths</h3>
                <div class="form-group">
                    <label>Data Directory</label>
                    <input type="text" id="setup-data-dir" value="/tmp/bitcell/data" placeholder="/path/to/bitcell/data">
                    <small>Directory where node data will be stored</small>
                </div>
                <div class="form-group">
                    <label>Config Path</label>
                    <input type="text" id="setup-config-path" value=".bitcell/admin/config.json" placeholder=".bitcell/admin/config.json">
                    <small>Path to configuration file</small>
                </div>
                <div class="wizard-actions">
                    <button class="btn" onclick="nextWizardStep(1)">Next →</button>
                </div>
            </div>

            <!-- Step 2: Deployment Configuration -->
            <div id="wizard-step-2" class="wizard-step">
                <h3 style="color: #00ffaa; margin-bottom: 1rem;">🚀 Deploy Network</h3>
                <p style="color: rgba(0, 255, 170, 0.7); margin-bottom: 1rem; font-size: 0.9rem;">
                    Automatically deploy local nodes to start your private network.
                </p>

                <div class="form-group">
                    <label>Validators</label>
                    <input type="number" id="deploy-validators" value="2" min="1" max="10">
                    <small>Number of validator nodes to deploy</small>
                </div>
                <div class="form-group">
                    <label>Miners</label>
                    <input type="number" id="deploy-miners" value="1" min="0" max="10">
                    <small>Number of miner nodes to deploy</small>
                </div>
                <div class="form-group">
                    <label>Full Nodes</label>
                    <input type="number" id="deploy-fullnodes" value="0" min="0" max="5">
                    <small>Number of non-validating full nodes</small>
                </div>
                
                <div class="wizard-actions">
                    <button class="btn btn-secondary" onclick="prevWizardStep(2)">← Back</button>
                    <button class="btn" onclick="nextWizardStep(2)">Deploy & Finish →</button>
                </div>
            </div>

            <!-- Step 3: Completion -->
            <div id="wizard-step-3" class="wizard-step">
                <div class="success-message">
                    <div class="success-icon">✅</div>
                    <h3>Setup Complete!</h3>
                    <p style="color: rgba(0, 255, 170, 0.7); margin-bottom: 2rem;">
                        Your BitCell admin console is now configured and ready to use.
                    </p>
                    <button class="btn" onclick="closeWizard()">Go to Dashboard →</button>
                </div>
            </div>
        </div>
    </div>

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
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h2 style="margin: 0;">🖥️ Registered Nodes</h2>
                <button class="btn" onclick="showDeployDialog()">+ Deploy Nodes</button>
            </div>
            <div id="nodes-list" class="node-list">
                <div class="loading">Loading nodes...</div>
            </div>
        </div>

        <!-- Deploy Dialog -->
        <div id="deploy-overlay" class="wizard-overlay">
            <div class="wizard-container" style="max-width: 500px;">
                <div class="wizard-header">
                    <h2>Deploy New Nodes</h2>
                    <p>Deploy new BitCell nodes to your network</p>
                </div>
                <div class="form-group">
                    <label>Node Type</label>
                    <select id="deploy-node-type" class="btn" style="width: 100%; padding: 0.75rem;">
                        <option value="validator">Validator</option>
                        <option value="miner">Miner</option>
                        <option value="fullnode">Full Node</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Number of Nodes</label>
                    <input type="number" id="deploy-count" value="1" min="1" max="10" style="width: 100%; padding: 0.75rem;">
                </div>
                <div class="form-group">
                    <label style="display: flex; align-items: center; gap: 0.5rem; cursor: pointer;">
                        <input type="checkbox" id="deploy-enable-dht" style="width: auto;" checked>
                        <span>Enable DHT Peer Discovery</span>
                    </label>
                </div>
                <div class="form-group" id="deploy-dht-options">
                    <label>Bootstrap Nodes (comma-separated multiaddrs)</label>
                    <input type="text" id="deploy-bootstrap" placeholder="/ip4/127.0.0.1/tcp/19000" style="width: 100%; padding: 0.75rem;">
                    <label style="margin-top: 0.5rem; display: block;">Key Seed (optional, for deterministic keys)</label>
                    <input type="text" id="deploy-key-seed" placeholder="e.g., bootstrap, miner1" style="width: 100%; padding: 0.75rem;">
                </div>
                <div class="wizard-actions">
                    <button class="btn btn-secondary" onclick="closeDeployDialog()">Cancel</button>
                    <button class="btn" onclick="deployNodes()">Deploy</button>
                </div>
            </div>
        </div>

        <!-- Log Viewer Modal -->
        <div id="log-modal" class="wizard-overlay" style="display: none;">
            <div class="wizard-container" style="max-width: 900px; max-height: 80vh;">
                <div class="wizard-header">
                    <h2>📋 Node Logs: <span id="log-node-id"></span></h2>
                    <button onclick="closeLogModal()" class="btn btn-secondary" style="position: absolute; right: 1.5rem; top: 1.5rem;">×</button>
                </div>
                <div class="log-controls" style="display: flex; gap: 10px; margin-bottom: 15px; padding: 10px; background: rgba(0, 255, 136, 0.05); border-radius: 8px;">
                    <select id="log-lines" onchange="refreshLogs()" class="btn" style="padding: 0.5rem;">
                        <option value="50">Last 50 lines</option>
                        <option value="100" selected>Last 100 lines</option>
                        <option value="500">Last 500 lines</option>
                        <option value="1000">Last 1000 lines</option>
                    </select>
                    <button onclick="refreshLogs()" class="btn">🔄 Refresh</button>
                </div>
                <pre id="log-content" class="log-output" style="background: #000; color: #0f8; padding: 15px; border-radius: 8px; font-family: 'Courier New', monospace; font-size: 12px; line-height: 1.4; max-height: 500px; overflow-y: auto; white-space: pre-wrap; word-wrap: break-word;"></pre>
            </div>
        </div>

        <!-- Blocks Section -->
        <div class="card" style="margin-top: 2rem;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h2 style="margin: 0;">⛓️ Recent Blocks</h2>
                <button class="btn" onclick="loadBlocks()">🔄 Refresh</button>
            </div>
            <div id="blocks-list" class="node-list">
                <div class="loading">Loading blocks...</div>
            </div>
        </div>

        <!-- Block Detail Modal -->
        <div id="block-modal" class="wizard-overlay" style="display: none;">
            <div class="wizard-container" style="max-width: 1200px; max-height: 90vh; overflow-y: auto;">
                <div class="wizard-header">
                    <h2>📦 Block <span id="block-height-title"></span></h2>
                    <button onclick="closeBlockModal()" class="btn btn-secondary" style="position: absolute; right: 1.5rem; top: 1.5rem;">×</button>
                </div>
                
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 2rem;">
                    <div>
                        <div class="metric">
                            <span class="metric-label">Block Hash</span>
                            <span id="block-hash" style="font-size: 0.9rem; font-family: monospace; color: #00ffaa;"></span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">Proposer</span>
                            <span id="block-proposer" style="font-size: 0.9rem; color: #00ffaa;"></span>
                        </div>
                    </div>
                    <div>
                        <div class="metric">
                            <span class="metric-label">Timestamp</span>
                            <span id="block-timestamp" style="font-size: 0.9rem; color: #00ffaa;"></span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">Transactions</span>
                            <span id="block-tx-count" style="font-size: 0.9rem; color: #00ffaa;"></span>
                        </div>
                    </div>
                </div>

                <h3 style="color: #00ffaa; margin-bottom: 1rem;">⚔️ Tournament Battle</h3>
                <div id="block-battle-container">
                    <div class="loading">Loading battle data...</div>
                </div>

                <div id="block-battle-viz" style="display: none;">
                    <div style="display: grid; grid-template-columns: 300px 1fr; gap: 2rem;">
                        <div>
                            <div class="metric">
                                <span class="metric-label">Glider A</span>
                                <span id="battle-glider-a" style="font-size: 0.9rem; color: #00ffaa;"></span>
                            </div>
                            <div class="metric">
                                <span class="metric-label">Glider B</span>
                                <span id="battle-glider-b" style="font-size: 0.9rem; color: #00ffaa;"></span>
                            </div>
                            <div class="metric">
                                <span class="metric-label">Winner</span>
                                <span id="battle-winner" style="font-size: 0.9rem; color: #00ff00; font-weight: bold;"></span>
                            </div>
                            <div class="metric">
                                <span class="metric-label">Energy A</span>
                                <span id="battle-energy-a" style="font-size: 0.9rem; color: #00ffaa;"></span>
                            </div>
                            <div class="metric">
                                <span class="metric-label">Energy B</span>
                                <span id="battle-energy-b" style="font-size: 0.9rem; color: #00ffaa;"></span>
                            </div>
                            <div style="margin-top: 1rem;">
                                <button id="play-pause-btn" class="btn" onclick="togglePlayPause()" style="width: 100%; margin-bottom: 0.5rem;">▶️ Play</button>
                                <div style="margin-top: 1rem;">
                                    <label style="display: block; margin-bottom: 0.5rem; color: #94a3b8;">Frame: <span id="current-frame">0</span> / <span id="total-frames">0</span></label>
                                    <input type="range" id="frame-slider" min="0" max="0" value="0" style="width: 100%;" oninput="seekFrame(this.value)">
                                </div>
                                <div style="margin-top: 1rem;">
                                    <label style="display: block; margin-bottom: 0.5rem; color: #94a3b8;">Speed</label>
                                    <input type="range" id="speed-slider" min="1" max="10" value="5" style="width: 100%;">
                                </div>
                            </div>
                        </div>
                        <div>
                            <canvas id="block-battle-canvas" width="512" height="512" style="border: 2px solid #00ffaa; border-radius: 4px; image-rendering: pixelated; width: 100%; max-width: 512px;"></canvas>
                            <div style="margin-top: 0.5rem; text-align: center; color: rgba(0, 255, 170, 0.6); font-size: 0.8rem;">
                                Step: <span id="battle-step">0</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Battle Visualization Section -->
        <div class="card" style="margin-top: 2rem;">
            <h2>⚔️ Cellular Automata Battle Visualization</h2>
            <div style="display: grid; grid-template-columns: 300px 1fr; gap: 2rem; margin-top: 1rem;">
                <div>
                    <h3 style="font-size: 1rem; margin-bottom: 1rem;">Battle Configuration</h3>
                    <div style="margin-bottom: 1rem;">
                        <label style="display: block; margin-bottom: 0.5rem; color: #94a3b8;">Glider A</label>
                        <select id="glider-a" class="btn" style="width: 100%; padding: 0.75rem;">
                            <option value="standard">Standard</option>
                            <option value="lightweight">Lightweight</option>
                            <option value="middleweight">Middleweight</option>
                            <option value="heavyweight">Heavyweight</option>
                        </select>
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <label style="display: block; margin-bottom: 0.5rem; color: #94a3b8;">Glider B</label>
                        <select id="glider-b" class="btn" style="width: 100%; padding: 0.75rem;">
                            <option value="standard">Standard</option>
                            <option value="lightweight">Lightweight</option>
                            <option value="middleweight">Middleweight</option>
                            <option value="heavyweight">Heavyweight</option>
                        </select>
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <label style="display: block; margin-bottom: 0.5rem; color: #94a3b8;">Steps</label>
                        <input type="number" id="battle-steps" value="1000" min="100" max="5000" class="btn" style="width: 100%; padding: 0.75rem;">
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <label style="display: block; margin-bottom: 0.5rem; color: #94a3b8;">Frames</label>
                        <input type="number" id="battle-frames" value="20" min="5" max="100" class="btn" style="width: 100%; padding: 0.75rem;">
                    </div>
                    <button id="run-battle" class="btn" style="width: 100%; padding: 0.75rem;" onclick="runBattle()">Run Battle</button>
                    <div id="battle-status" style="margin-top: 1rem; padding: 1rem; background: #0f172a; border-radius: 8px; display: none;">
                        <div style="color: #94a3b8; font-size: 0.875rem;">Status: <span id="battle-status-text">Ready</span></div>
                        <div style="margin-top: 0.5rem;">
                            <div style="color: #3b82f6;">Winner: <span id="battle-winner">-</span></div>
                            <div style="margin-top: 0.25rem; font-size: 0.875rem;">Energy A: <span id="battle-energy-a">-</span></div>
                            <div style="font-size: 0.875rem;">Energy B: <span id="battle-energy-b">-</span></div>
                        </div>
                    </div>
                </div>
                <div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                        <h3 style="font-size: 1rem;">Visualization</h3>
                        <div style="display: flex; gap: 0.5rem;">
                            <button id="play-pause" class="btn" onclick="togglePlayback()" disabled>▶️ Play</button>
                            <input type="range" id="frame-slider" min="0" max="0" value="0" style="width: 200px;" disabled oninput="seekFrame(this.value)">
                            <span id="frame-display" style="color: #94a3b8;">Frame: 0/0</span>
                        </div>
                    </div>
                    <canvas id="battle-canvas" width="512" height="512" style="width: 100%; max-width: 512px; border: 2px solid #334155; border-radius: 8px; background: #000;"></canvas>
                    <div id="battle-legend" style="margin-top: 1rem; display: flex; gap: 1rem; justify-content: center;">
                        <div style="display: flex; align-items: center; gap: 0.5rem;">
                            <div style="width: 20px; height: 20px; background: #3b82f6; border-radius: 4px;"></div>
                            <span style="font-size: 0.875rem; color: #94a3b8;">Glider A Region</span>
                        </div>
                        <div style="display: flex; align-items: center; gap: 0.5rem;">
                            <div style="width: 20px; height: 20px; background: #ef4444; border-radius: 4px;"></div>
                            <span style="font-size: 0.875rem; color: #94a3b8;">Glider B Region</span>
                        </div>
                        <div style="display: flex; align-items: center; gap: 0.5rem;">
                            <div style="width: 20px; height: 20px; background: #22c55e; border-radius: 4px;"></div>
                            <span style="font-size: 0.875rem; color: #94a3b8;">High Energy</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Setup Wizard State
        let wizardNodes = [];
        let currentStep = 1;

        // Check if setup is complete on load
        async function checkSetupStatus() {
            try {
                const response = await fetch('/api/setup/status');
                const data = await response.json();

                if (!data.initialized) {
                    document.getElementById('wizard-overlay').classList.add('active');
                } else {
                    // Load existing nodes
                    wizardNodes = data.nodes || [];
                }
            } catch (error) {
                console.error('Failed to check setup status:', error);
                // Show wizard if we can't determine status
                document.getElementById('wizard-overlay').classList.add('active');
            }
        }

        // Wizard navigation
        async function nextWizardStep(step) {
            if (step === 1) {
                // Save paths
                const dataDir = document.getElementById('setup-data-dir').value;
                const configPath = document.getElementById('setup-config-path').value;

                Promise.all([
                    fetch('/api/setup/data-dir', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ path: dataDir })
                    }),
                    fetch('/api/setup/config-path', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ path: configPath })
                    })
                ]).then(() => {
                    goToStep(2);
                }).catch(error => {
                    console.error('Failed to save paths:', error);
                    alert('Failed to save configuration paths');
                });
            } else if (step === 2) {
                // Deploy nodes
                const validators = parseInt(document.getElementById('deploy-validators').value) || 0;
                const miners = parseInt(document.getElementById('deploy-miners').value) || 0;
                const fullnodes = parseInt(document.getElementById('deploy-fullnodes').value) || 0;

                const deployments = [];
                let firstValidatorPort = null;

                // Execute deployments
                const btn = document.querySelector('#wizard-step-2 .btn:last-child');
                const originalText = btn.textContent;
                btn.textContent = 'Deploying...';
                btn.disabled = true;

                try {
                    // 1. Deploy first validator (Bootstrap Node)
                    if (validators > 0) {
                        const res = await fetch('/api/deployment/deploy', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ 
                                node_type: 'validator', 
                                count: 1,
                                enable_dht: true,
                                bootstrap_nodes: [] 
                            })
                        });
                        
                        if (!res.ok) throw new Error('Failed to deploy bootstrap validator');
                        const data = await res.json();
                        
                        if (data.nodes && data.nodes.length > 0) {
                            firstValidatorPort = data.nodes[0].port;
                            // Wait a moment for it to start
                            await new Promise(r => setTimeout(r, 2000));
                        }
                    }

                    const bootstrapNodes = firstValidatorPort ? [`/ip4/127.0.0.1/tcp/${firstValidatorPort}`] : [];

                    // 2. Deploy remaining validators
                    if (validators > 1) {
                        deployments.push(
                            fetch('/api/deployment/deploy', {
                                method: 'POST',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({ 
                                    node_type: 'validator', 
                                    count: validators - 1,
                                    enable_dht: true,
                                    bootstrap_nodes: bootstrapNodes
                                })
                            }).then(res => {
                                if (!res.ok) throw new Error('Failed to deploy validators');
                                return res;
                            })
                        );
                    }

                    // 3. Deploy miners
                    if (miners > 0) {
                        deployments.push(
                            fetch('/api/deployment/deploy', {
                                method: 'POST',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({ 
                                    node_type: 'miner', 
                                    count: miners,
                                    enable_dht: true,
                                    bootstrap_nodes: bootstrapNodes
                                })
                            }).then(res => {
                                if (!res.ok) throw new Error('Failed to deploy miners');
                                return res;
                            })
                        );
                    }

                    // 4. Deploy full nodes
                    if (fullnodes > 0) {
                        deployments.push(
                            fetch('/api/deployment/deploy', {
                                method: 'POST',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({ 
                                    node_type: 'fullnode', 
                                    count: fullnodes,
                                    enable_dht: true,
                                    bootstrap_nodes: bootstrapNodes
                                })
                            }).then(res => {
                                if (!res.ok) throw new Error('Failed to deploy full nodes');
                                return res;
                            })
                        );
                    }
                    
                    // Wait for all remaining deployments
                    await Promise.all(deployments);

                    // Complete setup
                    await fetch('/api/setup/complete', { method: 'POST' });

                    btn.textContent = 'Success!';
                    // Wait for nodes to actually start
                    setTimeout(() => {
                        goToStep(3);
                        setTimeout(() => {
                            window.location.reload();
                        }, 5000);
                    }, 1000);

                } catch (error) {
                    console.error('Deployment failed:', error);
                    alert('Deployment failed: ' + error.message);
                    btn.textContent = originalText;
                    btn.disabled = false;
                    return;
                }
            }
        }

        function prevWizardStep(step) {
            goToStep(step - 1);
        }

        function goToStep(step) {
            // Hide all steps
            document.querySelectorAll('.wizard-step').forEach(el => {
                el.classList.remove('active');
            });

            // Show target step
            document.getElementById(`wizard-step-${step}`).classList.add('active');

            // Update progress indicators
            document.querySelectorAll('.progress-step').forEach(el => {
                const stepNum = parseInt(el.dataset.step);
                el.classList.remove('active', 'completed');

                if (stepNum < step) {
                    el.classList.add('completed');
                } else if (stepNum === step) {
                    el.classList.add('active');
                }
            });

            currentStep = step;
        }

        // Add node to wizard
        async function addNodeToWizard() {
            const id = document.getElementById('node-id').value;
            const type = document.getElementById('node-type').value;
            const metrics = document.getElementById('node-metrics').value;
            const rpc = document.getElementById('node-rpc').value;

            if (!id || !metrics || !rpc) {
                alert('Please fill in all fields');
                return;
            }

            try {
                const response = await fetch('/api/setup/node', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        id,
                        node_type: type,
                        metrics_endpoint: metrics,
                        rpc_endpoint: rpc
                    })
                });

                if (!response.ok) {
                    throw new Error('Failed to add node');
                }

                const node = await response.json();
                wizardNodes.push(node);
                updateWizardNodesList();

                // Clear form
                document.getElementById('node-id').value = '';
                document.getElementById('node-metrics').value = '';
                document.getElementById('node-rpc').value = '';
            } catch (error) {
                console.error('Failed to add node:', error);
                alert('Failed to add node');
            }
        }

        function updateWizardNodesList() {
            const list = document.getElementById('wizard-nodes-list');

            if (wizardNodes.length === 0) {
                list.innerHTML = '<p style="color: rgba(0, 255, 170, 0.5); text-align: center; padding: 1rem;">No nodes added yet</p>';
                return;
            }

            list.innerHTML = wizardNodes.map(node => `
                <div class="node-item-wizard">
                    <div>
                        <div class="node-id">${node.id}</div>
                        <div class="node-type">${node.node_type}</div>
                    </div>
                    <div style="font-size: 0.8rem; color: rgba(0, 255, 170, 0.5);">
                        <div>${node.metrics_endpoint}</div>
                    </div>
                </div>
            `).join('');
        }

        function closeWizard() {
            document.getElementById('wizard-overlay').classList.remove('active');
            // Reload page to show dashboard with data
            window.location.reload();
        }

        async function showDeployDialog() {
            // Smart prepopulation: suggest first running validator as bootstrap node
            try {
                const response = await fetch('/api/nodes');
                const data = await response.json();
                
                // Find first running validator (node_type is lowercase from API)
                const runningValidator = data.nodes.find(n => 
                    n.node_type === 'validator' && n.status === 'running'
                );
                
                if (runningValidator) {
                    // Prepopulate bootstrap field with validator's address
                    // Ensure we use the P2P port, not the metrics port (which is usually port + 1)
                    // The API returns the P2P port in the 'port' field, so this should be correct.
                    // However, we'll double check against known metrics ports just in case.
                    const bootstrapAddr = `/ip4/127.0.0.1/tcp/${runningValidator.port}`;
                    document.getElementById('deploy-bootstrap').value = bootstrapAddr;
                    document.getElementById('deploy-bootstrap').placeholder = `e.g., ${bootstrapAddr}`;
                } else {
                    // No running validator, suggest default
                    document.getElementById('deploy-bootstrap').value = '';
                    document.getElementById('deploy-bootstrap').placeholder = '/ip4/127.0.0.1/tcp/19000';
                }
            } catch (error) {
                console.error('Failed to prepopulate bootstrap nodes:', error);
            }
            
            document.getElementById('deploy-overlay').classList.add('active');
        }

        function closeDeployDialog() {
            document.getElementById('deploy-overlay').classList.remove('active');
        }

        async function deployNodes() {
            const nodeType = document.getElementById('deploy-node-type').value;
            const count = parseInt(document.getElementById('deploy-count').value);
            const enableDht = document.getElementById('deploy-enable-dht').checked;
            const bootstrap = document.getElementById('deploy-bootstrap').value;
            const keySeed = document.getElementById('deploy-key-seed').value;

            if (isNaN(count) || count < 1 || count > 10) {
                alert('Please enter a valid number between 1 and 10');
                return;
            }

            // Parse bootstrap nodes
            let bootstrapNodes = bootstrap
                ? bootstrap.split(',').map(s => s.trim()).filter(s => s.length > 0)
                : [];

            // Validate bootstrap ports - check if user is trying to connect to a metrics port
            try {
                const nodesRes = await fetch('/api/nodes');
                const nodesData = await nodesRes.json();
                
                // Build map of metrics ports
                const metricsPorts = new Set();
                nodesData.nodes.forEach(n => {
                    // Metrics port is typically port + 1
                    metricsPorts.add(n.port + 1);
                });

                let warned = false;
                bootstrapNodes = bootstrapNodes.map(addr => {
                    // Extract port from multiaddr /ip4/x.x.x.x/tcp/PORT
                    const match = addr.match(/\/tcp\/(\d+)/);
                    if (match) {
                        const port = parseInt(match[1]);
                        if (metricsPorts.has(port)) {
                            // This looks like a metrics port!
                            const correctPort = port - 1;
                            if (!warned) {
                                if (confirm(`Warning: Port ${port} appears to be a metrics port (HTTP), not a P2P port. \n\nDo you want to automatically correct it to ${correctPort}?`)) {
                                    return addr.replace(`/tcp/${port}`, `/tcp/${correctPort}`);
                                }
                                warned = true;
                            }
                        }
                    }
                    return addr;
                });
            } catch (e) {
                console.warn("Failed to validate ports:", e);
            }

            try {
                const response = await fetch('/api/deployment/deploy', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        node_type: nodeType,
                        count: count,
                        config: {
                            network: "testnet",
                            enable_dht: enableDht,
                            bootstrap_nodes: bootstrapNodes,
                            key_seed: keySeed || null
                        }
                    })
                });

                if (!response.ok) {
                    let errorMessage = 'Deployment failed';
                    const text = await response.text();
                    try {
                        const error = JSON.parse(text);
                        errorMessage = error.error || error.message || errorMessage;
                    } catch (e) {
                        // Avoid showing large HTML blobs; use a generic message if text looks like HTML
                        if (text && !/^<!doctype|^<html/i.test(text.trim())) {
                            errorMessage = text;
                        }
                    }
                    throw new Error(errorMessage);
                }

                const data = await response.json();
                alert(`Successfully deployed ${data.nodes_deployed} ${nodeType} node(s)`);
                closeDeployDialog();
                
                // Refresh nodes list after a short delay
                setTimeout(updateNodes, 1000);
            } catch (error) {
                console.error('Deployment failed:', error);
                alert('Deployment failed: ' + error.message);
            }
        }

        // Global metrics cache
        let latestMetrics = null;

        // Fetch and update metrics
        async function updateMetrics() {
            try {
                const response = await fetch('/api/metrics');
                if (!response.ok) {
                    // If service unavailable (no nodes), just clear metrics or ignore
                    console.debug('Metrics unavailable:', response.status);
                    return;
                }
                const data = await response.json();
                if (!data || !data.chain) return;
                latestMetrics = data; // Cache metrics

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
                
                // Also refresh nodes list if we have new metrics
                if (document.getElementById('nodes-list').children.length > 0) {
                    updateNodes();
                }
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

                nodesList.innerHTML = data.nodes.map(node => {
                    // Find metrics for this node
                    let dhtPeers = 0;
                    if (latestMetrics && latestMetrics.node_metrics) {
                        const nodeMetric = latestMetrics.node_metrics.find(m => m.node_id === node.id);
                        if (nodeMetric) {
                            dhtPeers = nodeMetric.dht_peer_count || nodeMetric.peer_count || 0;
                        }
                    }
                    
                    return `
                    <div class="node-item">
                        <div class="node-info">
                            <h3>${node.id}</h3>
                            <p>Type: ${node.node_type} | ${node.address}:${node.port}</p>
                            ${node.enable_dht ? `<p style="color: #00ffaa; font-size: 0.9rem;">🌐 DHT: ${dhtPeers} peer(s)</p>` : ''}
                        </div>
                        <div class="actions">
                            <span class="status status-${node.status}">${node.status}</span>
                            <button class="btn" onclick="viewNodeConfig('${node.id}')" title="View Configuration">⚙️</button>
                            <button class="btn" onclick="viewLogs('${node.id}')" title="View Logs">📋</button>
                            <button class="btn" onclick="startNode('${node.id}')" ${node.status === 'running' ? 'disabled' : ''}>Start</button>
                            <button class="btn btn-danger" onclick="stopNode('${node.id}')" ${node.status === 'stopped' ? 'disabled' : ''}>Stop</button>
                            <button class="btn btn-danger" onclick="deleteNode('${node.id}')" title="Delete Node">🗑️</button>
                        </div>
                    </div>
                `}).join('');
            } catch (error) {
                console.error('Failed to fetch nodes:', error);
            }
        }

        async function startNode(id) {
            try {
                const response = await fetch(`/api/nodes/${id}/start`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ config: null })
                });
                
                if (!response.ok) {
                    let errorMessage = 'Failed to start node';
                    try {
                        const error = await response.json();
                        errorMessage = error.error || errorMessage;
                    } catch (e) {
                        // If JSON parsing fails, use default message
                    }
                    throw new Error(errorMessage);
                }
                
                updateNodes();
            } catch (error) {
                console.error('Failed to start node:', error);
                alert('Failed to start node: ' + error.message);
            }
        }

        async function stopNode(id) {
            try {
                const response = await fetch(`/api/nodes/${id}/stop`, { method: 'POST' });
                
                if (!response.ok) {
                    let errorMessage = 'Failed to stop node';
                    try {
                        const error = await response.json();
                        errorMessage = error.error || errorMessage;
                    } catch (e) {
                        // If JSON parsing fails, use default message
                    }
                    throw new Error(errorMessage);
                }
                
                updateNodes();
            } catch (error) {
                console.error('Failed to stop node:', error);
                alert('Failed to stop node: ' + error.message);
            }
        }

        async function viewNodeConfig(id) {
            try {
                const response = await fetch(`/api/nodes/${id}`);
                if (!response.ok) throw new Error('Failed to fetch node config');
                
                const data = await response.json();
                const node = data.node;
                
                const configText = `
Node ID: ${node.id}
Type: ${node.node_type}
Address: ${node.address}:${node.port}
Status: ${node.status}
DHT Enabled: ${node.enable_dht ? 'Yes' : 'No'}
${node.enable_dht ? `DHT Peers: ${node.dht_peer_count || 0}` : ''}
${node.bootstrap_nodes && node.bootstrap_nodes.length > 0 ? `Bootstrap Nodes: ${node.bootstrap_nodes.join(', ')}` : ''}
${node.key_seed ? `Key Seed: ${node.key_seed}` : ''}
                `.trim();
                
                alert(configText);
            } catch (error) {
                console.error('Failed to view node config:', error);
                alert('Failed to view node configuration');
            }
        }

        async function deleteNode(id) {
            if (!confirm(`Are you sure you want to delete node "${id}"? This action cannot be undone.`)) {
                return;
            }
            
            try {
                const response = await fetch(`/api/nodes/${id}`, { method: 'DELETE' });
                
                if (!response.ok) {
                    let errorMessage = 'Failed to delete node';
                    try {
                        const error = await response.json();
                        errorMessage = error.error || errorMessage;
                    } catch (e) {
                        // If JSON parsing fails, use default message
                    }
                    throw new Error(errorMessage);
                }
                
                alert(`Node "${id}" deleted successfully`);
                updateNodes();
            } catch (error) {
                console.error('Failed to delete node:', error);
                alert('Failed to delete node: ' + error.message);
            }
        }

        function formatBytes(bytes) {
            if (bytes < 1024) return bytes + 'B';
            if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + 'KB';
            if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + 'MB';
            return (bytes / (1024 * 1024 * 1024)).toFixed(1) + 'GB';
        }

        // Log viewer functions
        let currentLogNodeId = null;

        async function viewLogs(nodeId) {
            currentLogNodeId = nodeId;
            document.getElementById('log-node-id').textContent = nodeId;
            document.getElementById('log-modal').style.display = 'flex';
            await refreshLogs();
        }

        async function refreshLogs() {
            if (!currentLogNodeId) return;
            
            const lines = document.getElementById('log-lines').value;
            const logContent = document.getElementById('log-content');
            
            try {
                logContent.textContent = 'Loading logs...';
                const response = await fetch(`/api/nodes/${currentLogNodeId}/logs?lines=${lines}`);
                
                if (!response.ok) {
                    throw new Error('Failed to fetch logs');
                }
                
                const logs = await response.text();
                logContent.textContent = logs || 'No logs available';
                
                // Auto-scroll to bottom
                logContent.scrollTop = logContent.scrollHeight;
            } catch (error) {
                console.error('Failed to fetch logs:', error);
                logContent.textContent = `Error: ${error.message}`;
            }
        }

        function closeLogModal() {
            document.getElementById('log-modal').style.display = 'none';
            currentLogNodeId = null;
        }

        function formatUptime(seconds) {
            const days = Math.floor(seconds / 86400);
            const hours = Math.floor((seconds % 86400) / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            if (days > 0) return `${days}d ${hours}h`;
            if (hours > 0) return `${hours}h ${minutes}m`;
            return `${minutes}m`;
        }

        // Battle visualization state
        let battleFrames = [];
        let currentFrame = 0;
        let isPlaying = false;
        let playbackInterval = null;

        async function runBattle() {
            const gliderA = document.getElementById('glider-a').value;
            const gliderB = document.getElementById('glider-b').value;
            const steps = parseInt(document.getElementById('battle-steps').value);
            const frames = parseInt(document.getElementById('battle-frames').value);

            document.getElementById('run-battle').disabled = true;
            document.getElementById('battle-status').style.display = 'block';
            document.getElementById('battle-status-text').textContent = 'Running simulation...';

            try {
                const response = await fetch('/api/test/battle/visualize', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        glider_a: gliderA,
                        glider_b: gliderB,
                        steps: steps,
                        frame_count: frames,
                        downsample_size: 128
                    })
                });

                const data = await response.json();

                battleFrames = data.frames;
                currentFrame = 0;

                document.getElementById('battle-status-text').textContent = 'Complete';
                document.getElementById('battle-winner').textContent = data.winner;
                document.getElementById('battle-energy-a').textContent = data.final_energy_a.toLocaleString();
                document.getElementById('battle-energy-b').textContent = data.final_energy_b.toLocaleString();

                document.getElementById('frame-slider').max = battleFrames.length - 1;
                document.getElementById('frame-slider').disabled = false;
                document.getElementById('play-pause').disabled = false;

                renderFrame(0);
            } catch (error) {
                console.error('Battle failed:', error);
                document.getElementById('battle-status-text').textContent = 'Error: ' + error.message;
            } finally {
                document.getElementById('run-battle').disabled = false;
            }
        }

        function renderFrame(frameIndex) {
            if (frameIndex < 0 || frameIndex >= battleFrames.length) return;

            const frame = battleFrames[frameIndex];
            const canvas = document.getElementById('battle-canvas');
            const ctx = canvas.getContext('2d');
            const grid = frame.grid;
            const size = grid.length;
            const cellSize = canvas.width / size;

            ctx.clearRect(0, 0, canvas.width, canvas.height);

            // Render grid
            for (let y = 0; y < size; y++) {
                for (let x = 0; x < size; x++) {
                    const energy = grid[y][x];
                    if (energy > 0) {
                        // Color cells based on position and energy
                        const normalizedX = x / size;
                        const intensity = Math.min(255, energy * 2);

                        // Left side = blue (Glider A), right side = red (Glider B)
                        if (normalizedX < 0.5) {
                            ctx.fillStyle = `rgb(${intensity * 0.2}, ${intensity * 0.5}, ${intensity})`;
                        } else {
                            ctx.fillStyle = `rgb(${intensity}, ${intensity * 0.2}, ${intensity * 0.2})`;
                        }

                        ctx.fillRect(x * cellSize, y * cellSize, cellSize, cellSize);
                    }
                }
            }

            // Update frame display
            document.getElementById('frame-display').textContent = `Frame: ${frameIndex + 1}/${battleFrames.length} (Step ${frame.step})`;
            document.getElementById('frame-slider').value = frameIndex;
            currentFrame = frameIndex;
        }

        function togglePlayback() {
            isPlaying = !isPlaying;
            const btn = document.getElementById('play-pause');

            if (isPlaying) {
                btn.textContent = '⏸️ Pause';
                playbackInterval = setInterval(() => {
                    currentFrame = (currentFrame + 1) % battleFrames.length;
                    renderFrame(currentFrame);
                }, 100); // 10 FPS
            } else {
                btn.textContent = '▶️ Play';
                if (playbackInterval) {
                    clearInterval(playbackInterval);
                    playbackInterval = null;
                }
            }
        }

        function seekFrame(value) {
            if (isPlaying) {
                togglePlayback();
            }
            renderFrame(parseInt(value));
        }

        // Block visualization state
        let blockBattleFrames = [];
        let blockCurrentFrame = 0;
        let blockIsPlaying = false;
        let blockPlaybackInterval = null;

        async function loadBlocks() {
            const blocksList = document.getElementById('blocks-list');
            
            try {
                blocksList.innerHTML = '<div class="loading">Loading blocks...</div>';
                
                const response = await fetch('/api/blocks');
                if (!response.ok) {
                    throw new Error('Failed to fetch blocks');
                }
                
                const data = await response.json();
                
                if (data.blocks.length === 0) {
                    blocksList.innerHTML = '<div style="text-align: center; padding: 2rem; color: rgba(0, 255, 170, 0.5);">No blocks found</div>';
                    return;
                }
                
                blocksList.innerHTML = data.blocks.map(block => `
                    <div class="node-item" onclick="showBlockDetail(${block.height})" style="cursor: pointer;">
                        <div class="node-info">
                            <h3>Block #${block.height}</h3>
                            <p>Hash: ${block.hash} • Proposer: ${block.proposer}</p>
                            <p>Timestamp: ${new Date(block.timestamp * 1000).toLocaleString()}</p>
                        </div>
                        <div class="actions">
                            <span class="status status-running">${block.battle_count} Battle${block.battle_count !== 1 ? 's' : ''}</span>
                        </div>
                    </div>
                `).join('');
            } catch (error) {
                console.error('Failed to load blocks:', error);
                blocksList.innerHTML = '<div style="text-align: center; padding: 2rem; color: #ff0064;">Failed to load blocks</div>';
            }
        }

        async function showBlockDetail(height) {
            const modal = document.getElementById('block-modal');
            modal.style.display = 'flex';
            
            document.getElementById('block-height-title').textContent = `#${height}`;
            document.getElementById('block-battle-container').style.display = 'block';
            document.getElementById('block-battle-viz').style.display = 'none';
            
            try {
                // Fetch block details
                const blockResponse = await fetch(`/api/blocks/${height}`);
                if (!blockResponse.ok) throw new Error('Failed to fetch block details');
                const block = await blockResponse.json();
                
                document.getElementById('block-hash').textContent = block.hash;
                document.getElementById('block-proposer').textContent = block.proposer;
                document.getElementById('block-timestamp').textContent = new Date(block.timestamp * 1000).toLocaleString();
                document.getElementById('block-tx-count').textContent = block.transactions.length;
                
                // Fetch battle visualization
                const battleResponse = await fetch(`/api/blocks/${height}/battles`);
                if (!battleResponse.ok) throw new Error('Failed to fetch battle data');
                const battles = await battleResponse.json();
                
                if (battles.length > 0) {
                    const battle = battles[0]; // Show first battle
                    
                    document.getElementById('battle-glider-a').textContent = battle.glider_a_pattern;
                    document.getElementById('battle-glider-b').textContent = battle.glider_b_pattern;
                    document.getElementById('battle-winner').textContent = battle.winner.replace('_', ' ').toUpperCase();
                    
                    blockBattleFrames = battle.frames;
                    blockCurrentFrame = 0;
                    
                    document.getElementById('total-frames').textContent = blockBattleFrames.length;
                    document.getElementById('frame-slider').max = blockBattleFrames.length - 1;
                    document.getElementById('frame-slider').value = 0;
                    
                    document.getElementById('block-battle-container').style.display = 'none';
                    document.getElementById('block-battle-viz').style.display = 'block';
                    
                    renderBlockFrame(0);
                } else {
                    document.getElementById('block-battle-container').innerHTML = '<div style="text-align: center; padding: 2rem; color: rgba(0, 255, 170, 0.5);">No battle data available</div>';
                }
            } catch (error) {
                console.error('Failed to load block details:', error);
                document.getElementById('block-battle-container').innerHTML = `<div style="text-align: center; padding: 2rem; color: #ff0064;">Error: ${error.message}</div>`;
            }
        }

        function closeBlockModal() {
            document.getElementById('block-modal').style.display = 'none';
            if (blockIsPlaying) {
                togglePlayPause();
            }
        }

        function renderBlockFrame(frameIndex) {
            if (!blockBattleFrames || blockBattleFrames.length === 0) return;
            
            const frame = blockBattleFrames[frameIndex];
            const canvas = document.getElementById('block-battle-canvas');
            const ctx = canvas.getContext('2d');
            const grid = frame.grid;
            const size = grid.length;
            const cellSize = canvas.width / size;

            ctx.clearRect(0, 0, canvas.width, canvas.height);

            // Render grid with Game of Life cells
            for (let y = 0; y < size; y++) {
                for (let x = 0; x < size; x++) {
                    const energy = grid[y][x];
                    if (energy > 0) {
                        // Color cells based on position and energy
                        const normalizedX = x / size;
                        const intensity = Math.min(255, energy * 2);

                        // Left side = cyan (Glider A), right side = magenta (Glider B)
                        if (normalizedX < 0.5) {
                            ctx.fillStyle = `rgb(0, ${intensity}, ${intensity})`;
                        } else {
                            ctx.fillStyle = `rgb(${intensity}, 0, ${intensity})`;
                        }

                        ctx.fillRect(x * cellSize, y * cellSize, cellSize, cellSize);
                    }
                }
            }

            // Update displays
            document.getElementById('current-frame').textContent = frameIndex + 1;
            document.getElementById('battle-step').textContent = frame.step;
            document.getElementById('battle-energy-a').textContent = frame.energy_a.toLocaleString();
            document.getElementById('battle-energy-b').textContent = frame.energy_b.toLocaleString();
            document.getElementById('frame-slider').value = frameIndex;
            blockCurrentFrame = frameIndex;
        }

        function togglePlayPause() {
            blockIsPlaying = !blockIsPlaying;
            const btn = document.getElementById('play-pause-btn');

            if (blockIsPlaying) {
                btn.textContent = '⏸️ Pause';
                const speed = parseInt(document.getElementById('speed-slider').value);
                const delay = 1000 / speed; // Convert speed to delay
                
                blockPlaybackInterval = setInterval(() => {
                    blockCurrentFrame = (blockCurrentFrame + 1) % blockBattleFrames.length;
                    renderBlockFrame(blockCurrentFrame);
                }, delay);
            } else {
                btn.textContent = '▶️ Play';
                if (blockPlaybackInterval) {
                    clearInterval(blockPlaybackInterval);
                    blockPlaybackInterval = null;
                }
            }
        }

        function seekFrame(value) {
            if (blockIsPlaying) {
                togglePlayPause();
            }
            renderBlockFrame(parseInt(value));
        }

        // Initial load and auto-refresh
        checkSetupStatus();
        updateMetrics();
        updateNodes();
        loadBlocks();
        
        // DHT checkbox toggle handler
        document.getElementById('deploy-enable-dht').addEventListener('change', function() {
            const dhtOptions = document.getElementById('deploy-dht-options');
            dhtOptions.style.display = this.checked ? 'block' : 'none';
        });
        
        setInterval(updateMetrics, 5000);
        setInterval(updateNodes, 10000);
    </script>
</body>
</html>
    "#;

    (StatusCode::OK, Html(html))
}
