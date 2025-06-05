# CORS Proxy for Fermyon Spin

A lightweight, high-performance CORS proxy built for Fermyon Spin that forwards HTTP 
requests while adding permissive CORS headers and prevents caching of stale data.

## Features

- 🚀 **Fast & Lightweight**: Built with Rust and WebAssembly for optimal performance
- 🔄 **Full Request Forwarding**: Supports all HTTP methods (GET, POST, PUT, DELETE, etc.)
- 🌐 **Permissive CORS**: Adds appropriate CORS headers for browser compatibility
- 🚫 **No Caching**: Ensures all requests are passed through without caching
- 🔒 **URL Validation**: Basic security checks for target URLs
- 📦 **Easy Deployment**: Deploy to Fermyon Cloud or self-hosted Spin

## Prerequisites

- [Rust](https://rust-lang.org) (latest stable)
- [Fermyon Spin](https://spin.fermyon.dev) v3.0+
- `wasm32-wasip2` target: `rustup target add wasm32-wasip2`

## Quick Start

### 1. Clone and Build

```bash
git clone https://github.com/fschutt/corsproxy
cd corsproxy
spin build
```

### 2. Local Development

```bash
# Run locally
spin up

# Your proxy will be available at:
# http://localhost:3000
```

Now, adjust the values in spin.toml for allowed outbound hosts and test with:

```
curl -H "x-target-url: https://www.google.com" http://localhost:3000
```

Now, you have your own, private proxy for deployment!

### 3. Deploy to Fermyon Cloud

```bash
# Login to Fermyon Cloud
spin login

# Deploy
spin deploy
```

Now, note the target url and test again with:

```
curl -H "x-target-url: https://www.google.com" https://your-prod-url.fermyon.dev
```

## Usage

The proxy accepts target URLs via two methods:

### Method 1: Header-based (Recommended)

```bash
curl -H "x-target-url: https://api.example.com/data" \
     https://your-proxy.spin.app/
```

### Method 2: Query Parameter

```bash
# Simple URL
curl "https://your-proxy.spin.app/?url=https://api.example.com/data"

# URL-encoded (for complex URLs with query parameters)
curl "https://your-proxy.spin.app/?url=https%3A//api.example.com/search%3Fq%3Dhello%20world"
```

### JavaScript Example

```javascript
// Using fetch with header method
const response = await fetch('https://your-proxy.spin.app/', {
  method: 'GET',
  headers: {
    'x-target-url': 'https://api.example.com/data'
  }
});

// Using query parameter method
const response = await fetch(
  'https://your-proxy.spin.app/?url=' + 
  encodeURIComponent('https://api.example.com/data')
);
```

## Configuration

### Environment Variables

The proxy can be configured through the `spin.toml` file:

```toml
[component.cors-proxy]
# Allow outbound requests to any host
allowed_outbound_hosts = ["*"]

# Or restrict to specific domains
# allowed_outbound_hosts = ["https://api.example.com", "https://another-api.com"]
```

### Custom Headers

The proxy automatically adds these CORS headers:

- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD`
- `Access-Control-Allow-Headers: *`
- `Access-Control-Expose-Headers: *`
- `Access-Control-Max-Age: 86400` (for preflight requests)

And these cache prevention headers:

- `Cache-Control: no-store, no-cache, must-revalidate, proxy-revalidate`
- `Pragma: no-cache`
- `Expires: 0`

## Security Considerations

⚠️ **Important Security Notes:**

1. **Private Deployment**: This proxy is designed for private use. The URL is your security perimeter.
2. **No Rate Limiting**: There's no built-in rate limiting since this is intended for private use.
3. **URL Validation**: Only `http://` and `https://` URLs are accepted.
4. **Hop-by-hop Headers**: Connection-level headers are automatically filtered out.

### Recommended Security Practices

- Deploy to a private environment or use authentication
- Restrict `allowed_outbound_hosts` in production
- Monitor usage and implement rate limiting if needed
- Use HTTPS for your proxy deployment

## Development

### Testing

```bash
# Start the proxy locally
spin up

# Test with curl
curl -v -H "x-target-url: https://httpbin.org/get" \
     http://localhost:3000/

# Test preflight request
curl -v -X OPTIONS \
     -H "Access-Control-Request-Method: POST" \
     -H "Access-Control-Request-Headers: Content-Type" \
     http://localhost:3000/
```

### Debugging

Enable verbose logging:

```bash
RUST_LOG=debug spin up
```

## Deployment Options

### Fermyon Cloud

```bash
spin cloud deploy
```

### Self-hosted Spin

```bash
# Using Docker
docker run -p 3000:80 -v $(pwd):/app fermyon/spin:latest

# Or install Spin directly
spin up --listen 0.0.0.0:3000
```

### Custom Domain

Update `spin.toml` for custom domains:

```toml
[application.trigger.http]
base = "/"

# Add custom domain configuration as needed
```

## API Reference

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `*` | `/` | Proxy endpoint - forwards to target URL |
| `OPTIONS` | `/` | Handles CORS preflight requests |

### Headers

| Header | Description | Required |
|--------|-------------|----------|
| `x-target-url` | Target URL to proxy to | Yes (if not using query param) |

### Query Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `url` | URL-encoded target URL | Yes (if not using header) |

### Response Codes

| Code | Description |
|------|-------------|
| `200` | Successful proxy response |
| `500` | Proxy error (invalid URL, network error, etc.) |

## Troubleshooting

### Common Issues

**"Missing target URL" error:**
- Ensure you're providing either `x-target-url` header or `url` query parameter

**"Target URL must start with http://" error:**
- Only HTTP and HTTPS URLs are supported
- Check URL encoding if using query parameters

**Network errors:**
- Verify `allowed_outbound_hosts` in `spin.toml`
- Check if target server is accessible

**CORS issues:**
- The proxy adds permissive CORS headers automatically
- Check browser dev tools for specific CORS errors

### Performance Tips

- Use header method for better performance
- Keep URLs reasonably short
- The proxy doesn't cache, so consider caching on the client side if needed

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

[MIT License](LICENSE) - see LICENSE file for details

## Related

- [Fermyon Spin Documentation](https://spin.fermyon.dev/)
- [WebAssembly](https://webassembly.org/)
- [CORS Specification](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
