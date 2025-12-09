# BitCell Testnet Faucet

The BitCell testnet faucet provides automated token distribution for testing and development purposes.

## Features

- **Rate Limiting**: Configurable time-based and daily request limits per address
- **Anti-Abuse Protection**: 
  - Maximum recipient balance check
  - Address validation
  - Request tracking and audit logging
  - CAPTCHA support (configurable)
- **Web UI**: User-friendly interface for requesting tokens
- **API Endpoints**: RESTful API for integration
- **Real-time Statistics**: Track usage and distribution

## Configuration

The faucet is configured through the `FaucetConfig` struct:

```rust
use bitcell_admin::faucet::FaucetConfig;

let config = FaucetConfig {
    amount_per_request: 1_000_000_000,  // 1 CELL (in smallest units)
    rate_limit_seconds: 3600,           // 1 hour between requests
    max_requests_per_day: 5,            // Maximum 5 requests per day per address
    private_key: "0x...".to_string(),   // Faucet wallet private key
    node_rpc_host: "127.0.0.1".to_string(),
    node_rpc_port: 8545,
    require_captcha: true,              // Enable CAPTCHA verification
    max_recipient_balance: Some(10_000_000_000), // Max 10 CELL balance
};
```

### Environment Variables

You can also configure the faucet using environment variables:

```bash
export FAUCET_AMOUNT=1000000000
export FAUCET_RATE_LIMIT=3600
export FAUCET_MAX_REQUESTS_PER_DAY=5
export FAUCET_PRIVATE_KEY=0x...
export FAUCET_NODE_RPC_HOST=127.0.0.1
export FAUCET_NODE_RPC_PORT=8545
export FAUCET_REQUIRE_CAPTCHA=true
export FAUCET_MAX_RECIPIENT_BALANCE=10000000000
```

## Usage

### Enabling the Faucet

Add the faucet to your admin console:

```rust
use bitcell_admin::{AdminConsole, faucet::FaucetConfig};

let config = FaucetConfig {
    // ... your configuration
    private_key: std::env::var("FAUCET_PRIVATE_KEY")
        .expect("FAUCET_PRIVATE_KEY must be set"),
    ..Default::default()
};

let console = AdminConsole::new("127.0.0.1:8080".parse().unwrap())
    .with_faucet(config);

console.serve().await?;
```

### Web Interface

Once enabled, the faucet UI is available at:

```
http://localhost:8080/faucet
```

Users can:
1. Enter their BitCell address
2. Complete CAPTCHA (if enabled)
3. Request testnet tokens
4. View recent distributions

### API Endpoints

#### Request Tokens

```bash
POST /api/faucet/request
Content-Type: application/json

{
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
  "captcha_response": "optional_captcha_token"
}
```

Response:
```json
{
  "success": true,
  "message": "Successfully sent 1000000000 tokens to 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
  "tx_hash": "0xabc123...",
  "amount": 1000000000
}
```

#### Get Faucet Info

```bash
GET /api/faucet/info
```

Response:
```json
{
  "balance": 50000000000,
  "amount_per_request": 1000000000,
  "rate_limit_seconds": 3600,
  "max_requests_per_day": 5,
  "require_captcha": true
}
```

#### Check Eligibility

```bash
POST /api/faucet/check
Content-Type: application/json

{
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
}
```

Response:
```json
{
  "eligible": false,
  "message": "Rate limit active. Try again in 2400 seconds",
  "retry_after_seconds": 2400
}
```

#### Get Request History

```bash
GET /api/faucet/history
```

Returns the 50 most recent faucet distributions.

#### Get Statistics

```bash
GET /api/faucet/stats
```

Response:
```json
{
  "total_requests": 1234,
  "requests_last_hour": 12,
  "requests_last_day": 89,
  "total_distributed": 1234000000000
}
```

## Rate Limiting

The faucet implements two types of rate limiting:

1. **Time-based**: Minimum time between requests from the same address
2. **Daily limit**: Maximum number of requests per 24-hour period

Rate limits are tracked per address and reset automatically.

## Security Considerations

### Private Key Management

**⚠️ IMPORTANT**: The faucet private key should be:
- Stored securely (environment variables, secrets manager)
- Never committed to version control
- Limited to testnet funds only
- Rotated periodically

### CAPTCHA Integration

For production testnet deployments, integrate with a CAPTCHA service:

1. **reCAPTCHA**: Add Google reCAPTCHA widget to the web UI
2. **hCaptcha**: Alternative CAPTCHA provider
3. **Custom**: Implement your own challenge system

The faucet service checks `captcha_response` when `require_captcha` is enabled.

### Anti-Abuse

Additional anti-abuse measures:
- Maximum recipient balance check prevents excessive accumulation
- Request history tracking enables abuse detection
- IP-based rate limiting can be added at the reverse proxy level

## Monitoring

### Audit Logging

All faucet requests are logged with:
- Timestamp
- Recipient address
- Amount sent
- Transaction hash
- Status (completed/failed)

### Metrics

Track faucet health:
- Faucet balance (alert when low)
- Request rate (detect unusual patterns)
- Success/failure ratio
- Distribution by time period

### Alerts

Set up alerts for:
- Low faucet balance (< 10 CELL)
- High request rate (> 100/hour)
- Failed transaction ratio (> 5%)
- Repeated failures from same address

## Testing

Run faucet tests:

```bash
cargo test -p bitcell-admin faucet
```

Tests cover:
- Address validation
- Rate limiting (time-based and daily)
- Request statistics
- Error handling

## Troubleshooting

### "Faucet not enabled"

The faucet was not initialized. Ensure you call `.with_faucet(config)` when creating the AdminConsole.

### "Rate limit exceeded"

The address has made a request too recently. Wait for the cooldown period specified in the error message.

### "Faucet balance too low"

The faucet wallet needs to be refilled. Transfer testnet tokens to the faucet address.

### "Transaction failed"

Check:
- Node RPC connection
- Faucet private key is valid
- Faucet wallet has sufficient balance
- Gas fees are reasonable

## Example Deployment

```bash
# 1. Generate faucet wallet
# (use bitcell-wallet or any Ethereum-compatible wallet)

# 2. Fund the faucet wallet with testnet tokens
# Transfer ~100 CELL to the faucet address

# 3. Set environment variables
export FAUCET_PRIVATE_KEY="0x..."
export FAUCET_AMOUNT=1000000000
export FAUCET_RATE_LIMIT=3600

# 4. Run admin console with faucet
cargo run -p bitcell-admin --release

# 5. Access faucet UI
# Open http://localhost:8080/faucet in your browser
```

## Best Practices

1. **Testnet Only**: Never use mainnet funds in a faucet
2. **Rate Limits**: Set conservative limits to prevent abuse
3. **Monitoring**: Track usage and set up alerts
4. **CAPTCHA**: Always enable for public deployments
5. **Balance**: Keep faucet well-funded but not excessive
6. **Rotation**: Rotate faucet wallet periodically
7. **Logs**: Retain request logs for abuse investigation
8. **Documentation**: Provide clear usage instructions to users

## Future Enhancements

Potential improvements:
- [ ] GitHub OAuth integration
- [ ] Discord bot integration
- [ ] SMS verification
- [ ] Progressive request amounts (smaller for new users)
- [ ] Reputation system (trusted users get higher limits)
- [ ] Multi-faucet support (different networks)
- [ ] Admin dashboard for faucet management
- [ ] Automated refilling from treasury

## License

MIT OR Apache-2.0
