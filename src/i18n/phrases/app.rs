use std::collections::HashMap;

pub fn extend_zh_cn(m: &mut HashMap<String, String>) {
    for (key, value) in [
        ("Redis Desktop", "Redis Desktop"),
        ("Redis Workspace", "Redis 工作台"),
        (
            "Select a connection from the left, or create a new Redis connection first.",
            "从左侧选择一个连接，或先创建新的 Redis 连接。",
        ),
        (
            "Connection failed. Check the configuration and try again.",
            "连接失败，请检查连接配置后重试",
        ),
        ("Data", "数据"),
        ("Terminal", "终端"),
        ("Monitor", "监控"),
        ("Slow Log", "慢日志"),
        ("Clients", "客户端"),
        ("Scripts", "脚本"),
        ("Already up to date", "已是最新版本"),
        ("Failed to check for updates: ", "检查更新失败: "),
        (
            "Unable to initialize update checker",
            "无法初始化更新检查器",
        ),
        ("Loading connection...", "正在加载连接..."),
        ("Checking for updates...", "正在检查更新..."),
        ("Connecting to server...", "正在连接..."),
        ("Initializing connection...", "正在初始化连接..."),
    ] {
        m.insert(key.into(), value.into());
    }
}
