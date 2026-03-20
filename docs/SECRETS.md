# GitHub Actions Secrets 配置指南

本文档说明发布流程所需的 GitHub Secrets 配置。

## 必需的 Secrets

### macOS 签名和公证

| Secret 名称 | 说明 | 获取方式 |
|------------|------|----------|
| `APPLE_CERTIFICATES_P12` | Base64 编码的 Developer ID Application 证书 | 从 Keychain 导出 .p12 文件后 base64 编码 |
| `APPLE_CERTIFICATES_PASSWORD` | 导出证书时设置的密码 | 自行设置 |
| `APPLE_SIGNING_IDENTITY` | 签名身份名称 | 如 `Developer ID Application: Your Name (TEAM_ID)` |
| `APPLE_ID` | Apple ID 邮箱地址 | 你的 Apple Developer 账户邮箱 |
| `APPLE_TEAM_ID` | Apple 开发者团队 ID | 在 Apple Developer 网站查看 |
| `APPLE_APP_PASSWORD` | App-specific password | 在 appleid.apple.com 生成 |

## 配置步骤

### 1. 导出证书

```bash
# 从 Keychain 导出证书
# 1. 打开 Keychain Access
# 2. 找到 "Developer ID Application" 证书
# 3. 右键 -> 导出 -> 选择 .p12 格式
# 4. 设置密码（用于 APPLE_CERTIFICATES_PASSWORD）

# Base64 编码
base64 -i certificate.p12 -o certificate-base64.txt
```

### 2. 生成 App-Specific Password

1. 访问 https://appleid.apple.com
2. 登录你的 Apple ID
3. 在 "Security" 部分找到 "App-Specific Passwords"
4. 点击 "Generate Password"
5. 输入标签（如 "GitHub Actions"）
6. 复制生成的密码（用于 `APPLE_APP_PASSWORD`）

### 3. 获取 Team ID

1. 访问 https://developer.apple.com/account
2. 在 "Membership" 页面查看 "Team ID"

### 4. 配置 GitHub Secrets

在你的 GitHub 仓库中：

1. 进入 Settings -> Secrets and variables -> Actions
2. 点击 "New repository secret"
3. 逐个添加上述 Secrets

## 示例值

```
APPLE_SIGNING_IDENTITY: Developer ID Application: Your Name (ABCD1234)
APPLE_TEAM_ID: ABCD1234
APPLE_ID: your-email@example.com
APPLE_APP_PASSWORD: xxxx-xxxx-xxxx-xxxx
```

## 发布流程

配置完成后，发布新版本的步骤：

1. 更新 `Cargo.toml` 中的版本号
2. 更新 `Info.plist` 中的版本号
3. 提交更改并创建 tag：
   ```bash
   git add .
   git commit -m "chore: bump version to x.x.x"
   git tag vx.x.x
   git push origin main --tags
   ```
4. GitHub Actions 将自动构建并发布

## 发布产物

推送 tag 后，GitHub Actions 会构建以下产物：

| 平台 | 产物 |
|------|------|
| macOS (Intel) | `rust-redis-desktop-x86_64.dmg` |
| macOS (Apple Silicon) | `rust-redis-desktop-aarch64.dmg` |
| Windows | `rust-redis-desktop-x86_64-windows.zip` |
| Linux | `rust-redis-desktop-x86_64.AppImage` |
| Linux | `rust-redis-desktop_x.x.x_amd64.deb` |