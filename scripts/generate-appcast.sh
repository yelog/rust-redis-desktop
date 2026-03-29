#!/bin/bash

# 生成 Sparkle appcast.xml
# 用于 macOS 应用的自动更新

set -e

VERSION="${1:?需要提供版本号}"
X86_SIGNATURE="${2:-}"
X86_LENGTH="${3:-0}"
ARM_SIGNATURE="${4:-}"
ARM_LENGTH="${5:-0}"
IS_PRERELEASE="${6:-false}"
PUBLIC_KEY="${7:-}"

RELEASES_URL="https://github.com/yelog/rust-redis-desktop/releases/download/v${VERSION}"

# 获取当前日期
PUB_DATE=$(date -u +"%a, %d %b %Y %H:%M:%S GMT")

echo "生成 appcast.xml for version $VERSION"

# 开始生成 XML
cat > appcast.xml << 'XMLHEADER'
<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Rust Redis Desktop</title>
    <link>https://yelog.github.io/rust-redis-desktop/appcast.xml</link>
    <description>A Redis desktop manager written in Rust</description>
    <language>en</language>
XMLHEADER

# 添加 x86_64 版本
if [ -n "$X86_SIGNATURE" ]; then
cat >> appcast.xml << X86ITEM
    <item>
      <title>Version ${VERSION} (Intel)</title>
      <sparkle:version>${VERSION}</sparkle:version>
      <sparkle:shortVersionString>${VERSION}</sparkle:shortVersionString>
      <sparkle:minimumSystemVersion>10.15.0</sparkle:minimumSystemVersion>
      <pubDate>${PUB_DATE}</pubDate>
      <description><![CDATA[
        Rust Redis Desktop ${VERSION}
        
        A Redis desktop manager built with Rust.
        
        For Intel-based Macs.
      ]]></description>
      <enclosure url="${RELEASES_URL}/rust-redis-desktop-x86_64.dmg"
                 sparkle:edSignature="${X86_SIGNATURE}"
                 length="${X86_LENGTH}"
                 type="application/octet-stream"/>
    </item>
X86ITEM
fi

# 添加 aarch64 版本
if [ -n "$ARM_SIGNATURE" ]; then
cat >> appcast.xml << ARMITEM
    <item>
      <title>Version ${VERSION} (Apple Silicon)</title>
      <sparkle:version>${VERSION}</sparkle:version>
      <sparkle:shortVersionString>${VERSION}</sparkle:shortVersionString>
      <sparkle:minimumSystemVersion>11.0.0</sparkle:minimumSystemVersion>
      <pubDate>${PUB_DATE}</pubDate>
      <description><![CDATA[
        Rust Redis Desktop ${VERSION}
        
        A Redis desktop manager built with Rust.
        
        For Apple Silicon (M1/M2/M3) Macs.
      ]]></description>
      <enclosure url="${RELEASES_URL}/rust-redis-desktop-aarch64.dmg"
                 sparkle:edSignature="${ARM_SIGNATURE}"
                 length="${ARM_LENGTH}"
                 type="application/octet-stream"/>
    </item>
ARMITEM
fi

# 结束 XML
cat >> appcast.xml << 'XMLFOOTER'
  </channel>
</rss>
XMLFOOTER

echo "appcast.xml 已生成"
echo ""
echo "内容预览:"
head -20 appcast.xml