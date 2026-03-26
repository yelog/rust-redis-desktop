# Value Parsing Enhancement Design

## Overview

增强 Redis Desktop 的 value 解析能力，分四个阶段增量开发。

## Stage 1: New Serialization Formats (BSON, CBOR)

### Architecture

```
src/serialization/
├── mod.rs           # Add BSON, CBOR to SerializationFormat enum
├── bson.rs          # New: BSON parser
├── cbor.rs          # New: CBOR parser
└── ...              # Existing files
```

### Dependencies

```toml
bson = "2"
# CBOR: use ciborium or minicbor
```

### Implementation

**bson.rs**:
- `is_bson_serialization()` - Detect BSON format (check magic number)
- `parse_bson_to_json()` - Parse to JSON

**cbor.rs**:
- `is_cbor_serialization()` - Detect CBOR format
- `parse_cbor_to_json()` - Parse to JSON

**mod.rs changes**:
- Add `Bson`, `Cbor` to `SerializationFormat` enum
- Update `detect_serialization_format()` 
- Update `parse_to_json()`

### UI Updates

- Add `Bson`, `Cbor` to `BinaryFormat` enum
- Add detection and display logic

---

## Stage 2: New Formatters (YAML, TOML)

### Architecture

```
src/formatter/
├── mod.rs           # Add YAML, Toml to FormatterType enum
├── preset.rs        # Add YAML/TOML format functions
└── ...
```

### Dependencies

```toml
serde_yaml = "0.9"
toml = "0.8"
```

### Implementation

**preset.rs new functions**:

```rust
fn format_yaml(input: &[u8]) -> TransformResult {
    // Parse YAML -> convert to JSON -> format output
}

fn format_toml(input: &[u8]) -> TransformResult {
    // Parse TOML -> convert to JSON -> format output
}
```

**mod.rs changes**:
- Add `Yaml`, `Toml` to `FormatterType` enum
- Update `as_str()`, `from_str()`, `display_name()`

---

## Stage 3: Improve Kryo/FST Parsing

### Current Issues

Missing type support in `kryo.rs`:
- 0x02: Char
- 0x0D: ShortArray
- 0x0E: FloatArray
- 0x0F: DoubleArray
- Custom types return Unknown

### Improvements

**New types**:

```rust
pub enum KryoValue {
    // Existing types...
    
    // New
    ShortArray(Vec<i16>),
    FloatArray(Vec<f32>),
    DoubleArray(Vec<f64>),
}
```

**Improved detection**:
- Enhance `is_kryo_serialization()` to reduce false positives
- Add secondary validation

**FST improvements**:
- Parse more FST internal formats
- Better error handling

---

## Stage 4: Protobuf Schema Support

### Architecture

```
src/
├── serialization/
│   ├── protobuf.rs      # Refactor: support schema parsing
│   └── mod.rs           # Update
├── protobuf_schema/     # New module
│   ├── mod.rs           # Module entry
│   ├── parser.rs        # .proto file parsing
│   ├── registry.rs      # Schema registry
│   └── types.rs         # Schema type definitions
└── ui/
    └── proto_import.rs  # New: Proto file import UI
```

### Dependencies

```toml
prost = "0.12"
prost-types = "0.12"
protox = "0.6"
```

### Core Components

**ProtoSchemaRegistry**:

```rust
pub struct ProtoSchemaRegistry {
    schemas: HashMap<String, ProtoFile>,
    messages: HashMap<String, MessageDef>,
}

impl ProtoSchemaRegistry {
    pub fn import_file(path: &Path) -> Result<(), String>;
    pub fn find_message(&self, name: &str) -> Option<&MessageDef>;
    pub fn decode_with_schema(&self, data: &[u8], msg_name: &str) -> Result<JsonValue, String>;
}
```

### UI Flow

```
User clicks "Import Proto" button
    ↓
File dialog to select .proto file
    ↓
Parse file, show available message types
    ↓
User selects message type
    ↓
Decode binary data with schema
    ↓
Display JSON with field names
```

### Global State

```rust
pub static PROTO_REGISTRY: GlobalSignal<ProtoSchemaRegistry> = Signal::global(ProtoSchemaRegistry::new);
```

---

## Implementation Order

1. Stage 1: BSON + CBOR (simplest, immediate value)
2. Stage 2: YAML + TOML (simple)
3. Stage 3: Kryo/FST improvements (medium)
4. Stage 4: Protobuf Schema (most complex)

Each stage is independently testable and deployable.