mod parser;
mod registry;
mod types;

pub use parser::{parse_proto_content, parse_proto_file};
pub use registry::ProtoRegistry;
pub use types::{EnumDef, FieldDef, FieldLabel, FieldType, MessageDef, ProtoFile};

use dioxus::prelude::*;

pub static PROTO_REGISTRY: GlobalSignal<ProtoRegistry> = Signal::global(ProtoRegistry::new);
