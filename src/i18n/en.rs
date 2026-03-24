use std::collections::HashMap;

pub fn load() -> HashMap<String, String> {
    let mut m = HashMap::new();

    // Connection
    m.insert("app.title".into(), "Rust Redis Desktop".into());
    m.insert("connection.new".into(), "New Connection".into());
    m.insert("connection.name".into(), "Name".into());
    m.insert("connection.host".into(), "Host".into());
    m.insert("connection.port".into(), "Port".into());
    m.insert("connection.username".into(), "Username".into());
    m.insert("connection.password".into(), "Password".into());
    m.insert("connection.database".into(), "Database".into());
    m.insert("connection.test".into(), "Test".into());
    m.insert("connection.save".into(), "Save".into());
    m.insert("connection.cancel".into(), "Cancel".into());
    m.insert("connection.delete".into(), "Delete".into());
    m.insert("connection.edit".into(), "Edit".into());
    m.insert("connection.connecting".into(), "Connecting...".into());
    m.insert("connection.connected".into(), "Connected".into());
    m.insert("connection.disconnected".into(), "Disconnected".into());
    m.insert("connection.error".into(), "Error".into());
    m.insert("connection.mode.direct".into(), "Direct".into());
    m.insert("connection.mode.cluster".into(), "Cluster".into());
    m.insert("connection.mode.sentinel".into(), "Sentinel".into());
    m.insert("connection.ssh.enable".into(), "Enable SSH Tunnel".into());
    m.insert("connection.export".into(), "Export".into());
    m.insert("connection.import".into(), "Import".into());

    // Keys
    m.insert("key.search".into(), "Search Key...".into());
    m.insert("key.refresh".into(), "Refresh".into());
    m.insert("key.add".into(), "Add Key".into());
    m.insert("key.delete".into(), "Delete".into());
    m.insert("key.rename".into(), "Rename".into());
    m.insert("key.copy".into(), "Copy".into());
    m.insert("key.ttl".into(), "TTL".into());
    m.insert("key.type".into(), "Type".into());
    m.insert("key.size".into(), "Size".into());
    m.insert("key.value".into(), "Value".into());
    m.insert("key.no_data".into(), "No data".into());
    m.insert("key.pattern_delete".into(), "Pattern Delete".into());
    m.insert("key.memory_analysis".into(), "Memory Analysis".into());

    // Types
    m.insert("type.string".into(), "String".into());
    m.insert("type.hash".into(), "Hash".into());
    m.insert("type.list".into(), "List".into());
    m.insert("type.set".into(), "Set".into());
    m.insert("type.zset".into(), "Sorted Set".into());
    m.insert("type.stream".into(), "Stream".into());

    // Actions
    m.insert("action.save".into(), "Save".into());
    m.insert("action.cancel".into(), "Cancel".into());
    m.insert("action.confirm".into(), "Confirm".into());
    m.insert("action.delete".into(), "Delete".into());
    m.insert("action.close".into(), "Close".into());
    m.insert("action.copy".into(), "Copy".into());
    m.insert("action.paste".into(), "Paste".into());
    m.insert("action.select_all".into(), "Select All".into());

    // Messages
    m.insert("message.success".into(), "Success".into());
    m.insert("message.error".into(), "Error".into());
    m.insert("message.loading".into(), "Loading...".into());
    m.insert("message.saved".into(), "Saved".into());
    m.insert("message.deleted".into(), "Deleted".into());
    m.insert("message.copied".into(), "Copied to clipboard".into());

    // Settings
    m.insert("settings.title".into(), "Settings".into());
    m.insert("settings.theme".into(), "Theme".into());
    m.insert("settings.language".into(), "Language".into());
    m.insert("settings.general".into(), "General".into());
    m.insert("settings.about".into(), "About".into());

    // Stream
    m.insert("stream.consumer_groups".into(), "Consumer Groups".into());
    m.insert("stream.group.new".into(), "New Group".into());
    m.insert("stream.group.name".into(), "Group Name".into());
    m.insert("stream.group.consumers".into(), "Consumers".into());
    m.insert("stream.group.pending".into(), "Pending".into());
    m.insert("stream.consumer.name".into(), "Consumer Name".into());
    m.insert("stream.consumer.idle".into(), "Idle".into());

    // Dialog
    m.insert("dialog.confirm".into(), "Confirm".into());
    m.insert("dialog.delete_confirm".into(), "Are you sure?".into());
    m.insert(
        "dialog.delete_desc".into(),
        "This action cannot be undone".into(),
    );

    // Export/Import
    m.insert("export.title".into(), "Export".into());
    m.insert("import.title".into(), "Import".into());
    m.insert("export.format".into(), "Format".into());
    m.insert("export.success".into(), "Export successful".into());
    m.insert("import.success".into(), "Import successful".into());

    // Readonly
    m.insert("readonly.mode".into(), "Readonly Mode".into());
    m.insert(
        "readonly.warning".into(),
        "Connection is in readonly mode, write operations are blocked".into(),
    );

    m
}
