# Smart Home Remote Devices - Implementation Guide

This directory contains the complete technical documentation for implementing remote device support in the Smart Home project.

## Documents

### Main Architecture Document
- **[architecture.md](architecture.md)** - Complete technical architecture and implementation guide (comprehensive reference)

### Step-by-Step Implementation Guides

Follow these steps in order to implement the system:

1. **[Step 1: Protocol Design](step-1-protocol-design.md)**
   - Define binary TCP protocol for outlets (Request/Response enums)
   - Define binary UDP protocol for thermometers
   - Add `src/protocol/` module with serialization

2. **[Step 2: Add TCP Support to Outlet](step-2-outlet-remote.md)**
   - Add `OutletMode` (Local/Remote)
   - Implement TCP client in `Outlet`
   - Add connection management and error handling

3. **[Step 3: Add UDP Support to Thermometer](step-3-thermometer-remote.md)**
   - Add `ThermometerMode` (Local/Remote)
   - Implement background UDP receiver thread
   - Add thread lifecycle management with `Drop`

4. **[Step 4: Create Outlet Simulator](step-4-outlet-simulator.md)**
   - Create TCP server binary (`outlet_simulator`)
   - Implement multi-client support
   - Test outlet control over network

5. **[Step 5: Create Thermometer Simulator](step-5-thermometer-simulator.md)**
   - Create UDP sender binary (`thermometer_simulator`)
   - Add TOML configuration support
   - Implement realistic temperature generation

6. **[Step 6: Create Example Application](step-6-example-app.md)**
   - Build complete smart home example
   - Demonstrate full system integration
   - Add helper scripts for managing simulators

7. **[Step 7: Testing Strategy](step-7-testing.md)**
   - Unit tests for protocols
   - Integration tests for TCP/UDP
   - Mock simulators for automated testing

## Quick Start

### For Implementation

Start with Step 1 and work through sequentially:

```bash
# Read the first step
cat .claude/step-1-protocol-design.md

# Follow the implementation instructions
# Move to next step when complete
```

### For Reference

Use the architecture document for comprehensive reference:

```bash
# View complete architecture
cat .claude/architecture.md
```

## File Organization

```
.claude/
├── README.md                    # This file
├── architecture.md              # Complete reference document
├── step-1-protocol-design.md    # Binary protocols
├── step-2-outlet-remote.md      # TCP outlet implementation
├── step-3-thermometer-remote.md # UDP thermometer implementation
├── step-4-outlet-simulator.md   # TCP server
├── step-5-thermometer-simulator.md # UDP sender
├── step-6-example-app.md        # Full example
└── step-7-testing.md            # Testing strategy
```

## Key Features

### Binary Protocols
- **TCP for Outlets**: Type-safe `Request`/`Response` enums with bincode serialization
- **UDP for Thermometers**: Fixed 8-byte temperature readings with timestamps

### Dual Mode Support
- **Local Mode**: Direct in-memory access (backward compatible)
- **Remote Mode**: Network communication via TCP/UDP

### Simulators
- **Outlet Simulator**: Multi-client TCP server in separate threads
- **Thermometer Simulator**: UDP sender with configurable intervals

### Thread Safety
- `Arc<Mutex<T>>` for shared state
- Background threads with clean shutdown via `Drop`
- Channel-based signaling for thread communication

## Dependencies

Required Cargo dependencies:

```toml
[dependencies]
bincode = "1.3"          # Binary serialization
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"             # Random temperature generation
toml = "0.8"             # Configuration files
```

## Testing

Each step includes verification instructions:

```bash
# Run all tests
cargo test

# Run specific test
cargo test protocol

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_test
```

## Common Issues

### Build Errors
- **Clone trait**: Device enum can't be cloned with thread handles
- **Borrow checker**: Use `&mut self` or `RefCell` for interior mutability

### Runtime Errors
- **Address in use**: Kill existing simulators or use different ports
- **Connection refused**: Start simulators before running examples
- **Timeout**: Check firewall settings and network connectivity

## Best Practices

1. **Always read the prerequisites** for each step
2. **Test each step** before moving to the next
3. **Keep simulators running** when testing examples
4. **Use helper scripts** to manage multiple simulators
5. **Check logs** from both client and simulator sides

## Support

For questions or issues:
1. Check the specific step document
2. Review the architecture document
3. Look at the example code in `examples/`
4. Check test files in `tests/` for usage patterns

---

**Last Updated:** 2025-11-09
**Project:** Smart Home Remote Devices (hw3_smart_home_devices)
