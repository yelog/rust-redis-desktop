use std::collections::HashMap;

pub fn load() -> HashMap<String, String> {
    let mut m = HashMap::new();

    // 连接相关
    m.insert("app.title".into(), "Rust Redis Desktop".into());
    m.insert("connection.new".into(), "新建连接".into());
    m.insert("connection.name".into(), "连接名称".into());
    m.insert("connection.host".into(), "主机".into());
    m.insert("connection.port".into(), "端口".into());
    m.insert("connection.username".into(), "用户名".into());
    m.insert("connection.password".into(), "密码".into());
    m.insert("connection.database".into(), "数据库".into());
    m.insert("connection.test".into(), "测试连接".into());
    m.insert("connection.save".into(), "保存".into());
    m.insert("connection.cancel".into(), "取消".into());
    m.insert("connection.delete".into(), "删除".into());
    m.insert("connection.edit".into(), "编辑".into());
    m.insert("connection.connecting".into(), "连接中...".into());
    m.insert("connection.connected".into(), "已连接".into());
    m.insert("connection.disconnected".into(), "未连接".into());
    m.insert("connection.error".into(), "连接错误".into());
    m.insert("connection.mode.direct".into(), "直连".into());
    m.insert("connection.mode.cluster".into(), "集群".into());
    m.insert("connection.mode.sentinel".into(), "哨兵".into());
    m.insert("connection.ssh.enable".into(), "启用 SSH 隧道".into());
    m.insert("connection.export".into(), "导出连接".into());
    m.insert("connection.import".into(), "导入连接".into());

    // 键操作
    m.insert("key.search".into(), "搜索 Key...".into());
    m.insert("key.refresh".into(), "刷新".into());
    m.insert("key.add".into(), "添加 Key".into());
    m.insert("key.delete".into(), "删除".into());
    m.insert("key.rename".into(), "重命名".into());
    m.insert("key.copy".into(), "复制".into());
    m.insert("key.ttl".into(), "TTL".into());
    m.insert("key.type".into(), "类型".into());
    m.insert("key.size".into(), "大小".into());
    m.insert("key.value".into(), "值".into());
    m.insert("key.no_data".into(), "暂无数据".into());
    m.insert("key.pattern_delete".into(), "模式删除".into());
    m.insert("key.memory_analysis".into(), "内存分析".into());

    // 数据类型
    m.insert("type.string".into(), "字符串".into());
    m.insert("type.hash".into(), "哈希".into());
    m.insert("type.list".into(), "列表".into());
    m.insert("type.set".into(), "集合".into());
    m.insert("type.zset".into(), "有序集合".into());
    m.insert("type.stream".into(), "流".into());

    // 操作按钮
    m.insert("action.save".into(), "保存".into());
    m.insert("action.cancel".into(), "取消".into());
    m.insert("action.confirm".into(), "确认".into());
    m.insert("action.delete".into(), "删除".into());
    m.insert("action.close".into(), "关闭".into());
    m.insert("action.copy".into(), "复制".into());
    m.insert("action.paste".into(), "粘贴".into());
    m.insert("action.select_all".into(), "全选".into());

    // 消息提示
    m.insert("message.success".into(), "操作成功".into());
    m.insert("message.error".into(), "操作失败".into());
    m.insert("message.loading".into(), "加载中...".into());
    m.insert("message.saved".into(), "已保存".into());
    m.insert("message.deleted".into(), "已删除".into());
    m.insert("message.copied".into(), "已复制到剪贴板".into());

    // 设置
    m.insert("settings.title".into(), "设置".into());
    m.insert("settings.theme".into(), "主题".into());
    m.insert("settings.language".into(), "语言".into());
    m.insert("settings.general".into(), "常规".into());
    m.insert("settings.about".into(), "关于".into());

    // Stream 消费者组
    m.insert("stream.consumer_groups".into(), "消费者组".into());
    m.insert("stream.group.new".into(), "新建消费组".into());
    m.insert("stream.group.name".into(), "组名".into());
    m.insert("stream.group.consumers".into(), "消费者".into());
    m.insert("stream.group.pending".into(), "待处理".into());
    m.insert("stream.consumer.name".into(), "消费者名称".into());
    m.insert("stream.consumer.idle".into(), "空闲时间".into());

    // 确认对话框
    m.insert("dialog.confirm".into(), "确认".into());
    m.insert("dialog.delete_confirm".into(), "确定要删除吗？".into());
    m.insert("dialog.delete_desc".into(), "此操作不可撤销".into());

    // 导出导入
    m.insert("export.title".into(), "导出".into());
    m.insert("import.title".into(), "导入".into());
    m.insert("export.format".into(), "格式".into());
    m.insert("export.success".into(), "导出成功".into());
    m.insert("import.success".into(), "导入成功".into());

    // 只读模式
    m.insert("readonly.mode".into(), "只读模式".into());
    m.insert(
        "readonly.warning".into(),
        "当前连接为只读模式，写操作已被阻止".into(),
    );

    m
}
