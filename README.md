# reqly ⚡

> Postman for the terminal. A fast, terminal-native API client written in Rust.

**reqly** is a developer tool for making HTTP, GraphQL, and WebSocket requests directly from the terminal. It combines the speed of CLI tools like `curl` and `httpie` with an interactive, beautifully designed Terminal UI (TUI) inspired by tools like Postman.

## Features

- **Multi-Protocol Support:**
  - **HTTP:** Full REST support (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS).
  - **GraphQL:** Native support for queries, mutations, JSON variables, and operation names.
  - **WebSocket:** Live, interactive CLI messaging for WS/WSS endpoints.
- **Dual Interface:**
  - **CLI Mode:** Fast, scriptable, zero-friction command-line interface.
  - **TUI Mode:** Keyboard-driven interactive UI with dedicated protocol tabs.
- **Developer Experience:**
  - Automatic JSON pretty-printing.
  - **Local SQLite History:** Automatically saves your requests. Cycle through past URLs and bodies in-place perfectly mimicking terminal history.
  - Multi-line editing, Response scrolling, and clipboard copying.
  - Clean modular architecture designed for easy extensibility (e.g., adding gRPC in the future).
  - Built natively in Rust for blazing-fast performance.

---

## Installation

Ensure you have Rust and Cargo installed, then clone and build:

```bash
git clone https://github.com/ajkumar1205/reqly.git
cd reqly
cargo build --release
```

The executable will be available at `target/release/reqly`. You can move it to your PATH for global access:
```bash
sudo cp target/release/reqly /usr/local/bin/
```

Or just run it locally using `cargo run`.

---

## Interactive TUI Mode

Launch the TUI by running `reqly` without any arguments:

```bash
reqly
```

### TUI Keybindings
- **`F6`**: Next panel
- **`F7`** / **`Shift+Tab`**: Previous panel
- **`Tab`**: Insert 4 spaces inside multiline fields (Headers / Body / GraphQL) OR jump to next panel in single-line fields
- **`Ctrl+P`**: Cycle through protocols (HTTP → GraphQL → WebSocket)
- **`Up` / `Down`**: Navigate multi-line text or scroll response panes
- **`PageUp` / `PageDown`** (or `Ctrl+Up` / `Ctrl+Down`): Cycle backward/forward through local request history (in-place)
- **`y` / `c`**: Copy response body to clipboard (only when Response panel is focused)
- **`Enter`**: Send request / connect / resend
- **`Space`**: Cycle HTTP methods (when Method pane is focused)
- **`Ctrl+C`**: Quit

---

## CLI Mode

reqly is equally powerful as a standard command-line executable.

### HTTP

**GET Request:**
```bash
reqly GET https://jsonplaceholder.typicode.com/posts/1
```

**POST Request with body and custom headers:**
```bash
reqly POST https://httpbin.org/post \
  -H "Authorization: Bearer my-token" \
  -H "Content-Type: application/json" \
  -d '{"name": "reqly", "fast": true}'
```

### GraphQL

Send a GraphQL query with JSON variables:
```bash
reqly graphql https://countries.trevorblades.com/ \
  -q 'query GetCountry($code: ID!) { country(code: $code) { name capital } }' \
  -v '{"code": "IN"}' \
  -o GetCountry
```

### WebSocket

Start an interactive WebSocket session (type your messages and see server replies in real-time):
```bash
reqly ws wss://echo.websocket.org --json
```

---

## Testing

reqly comes with a comprehensive suite of unit and integration tests (using `wiremock` for local server mocking). Run the tests via:

```bash
cargo test
```

---

## License

MIT License.
