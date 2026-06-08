use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use crate::updater::UpdateInfo;
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReleaseNoteBlock {
    Heading { level: usize, text: String },
    Bullet(String),
    Numbered { number: String, text: String },
    Paragraph(String),
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum UpdateDialogState {
    #[default]
    Ready,
    Downloading,
    Completed,
    Error,
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{} MB", bytes / (1024 * 1024))
    }
}

fn progress_percent(downloaded: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }

    ((downloaded.saturating_mul(100) / total).min(100)) as u8
}

fn strip_inline_markdown(text: &str) -> String {
    text.replace("**", "")
        .replace("__", "")
        .replace('`', "")
        .trim()
        .to_string()
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }

    let rest = line.get(hashes..)?.trim_start();
    if rest.is_empty() || rest.len() == line.len() - hashes {
        return None;
    }

    Some((hashes, strip_inline_markdown(rest)))
}

fn parse_numbered_list(line: &str) -> Option<(String, String)> {
    let (number, rest) = line.split_once('.')?;
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let text = rest.trim_start();
    if text.is_empty() {
        return None;
    }

    Some((format!("{}.", number), strip_inline_markdown(text)))
}

fn parse_release_note_markdown(markdown: &str) -> Vec<ReleaseNoteBlock> {
    let mut blocks = Vec::new();
    let mut paragraph_lines = Vec::<String>::new();

    fn flush_paragraph(lines: &mut Vec<String>, blocks: &mut Vec<ReleaseNoteBlock>) {
        if lines.is_empty() {
            return;
        }

        blocks.push(ReleaseNoteBlock::Paragraph(strip_inline_markdown(
            &lines.join(" "),
        )));
        lines.clear();
    }

    for raw_line in markdown.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            flush_paragraph(&mut paragraph_lines, &mut blocks);
            continue;
        }

        if let Some((level, text)) = parse_heading(line) {
            flush_paragraph(&mut paragraph_lines, &mut blocks);
            blocks.push(ReleaseNoteBlock::Heading { level, text });
            continue;
        }

        if let Some(text) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            flush_paragraph(&mut paragraph_lines, &mut blocks);
            blocks.push(ReleaseNoteBlock::Bullet(strip_inline_markdown(text)));
            continue;
        }

        if let Some((number, text)) = parse_numbered_list(line) {
            flush_paragraph(&mut paragraph_lines, &mut blocks);
            blocks.push(ReleaseNoteBlock::Numbered { number, text });
            continue;
        }

        paragraph_lines.push(line.to_string());
    }

    flush_paragraph(&mut paragraph_lines, &mut blocks);
    blocks
}

fn render_release_note_block(
    block: ReleaseNoteBlock,
    colors: ThemeColors,
    index: usize,
) -> Element {
    match block {
        ReleaseNoteBlock::Heading { level, text } => {
            let font_size = match level {
                1 => "16px",
                2 => "15px",
                _ => "14px",
            };
            let margin_top = if index == 0 { "0" } else { "10px" };

            rsx! {
                div {
                    margin_top,
                    color: "{colors.text}",
                    font_size,
                    font_weight: "700",
                    line_height: "1.35",
                    "{text}"
                }
            }
        }
        ReleaseNoteBlock::Bullet(text) => rsx! {
            div {
                display: "flex",
                align_items: "flex-start",
                gap: "8px",
                color: "{colors.text}",
                font_size: "13px",
                line_height: "1.5",

                span {
                    color: "{colors.accent}",
                    font_weight: "700",
                    line_height: "1.5",
                    "•"
                }

                span {
                    flex: "1",
                    word_break: "break-word",
                    "{text}"
                }
            }
        },
        ReleaseNoteBlock::Numbered { number, text } => rsx! {
            div {
                display: "flex",
                align_items: "flex-start",
                gap: "8px",
                color: "{colors.text}",
                font_size: "13px",
                line_height: "1.5",

                span {
                    min_width: "22px",
                    color: "{colors.accent}",
                    font_weight: "700",
                    line_height: "1.5",
                    "{number}"
                }

                span {
                    flex: "1",
                    word_break: "break-word",
                    "{text}"
                }
            }
        },
        ReleaseNoteBlock::Paragraph(text) => rsx! {
            div {
                color: "{colors.text}",
                font_size: "13px",
                line_height: "1.5",
                word_break: "break-word",
                "{text}"
            }
        },
    }
}

#[component]
fn MarkdownReleaseNotes(markdown: String, colors: ThemeColors) -> Element {
    let blocks = parse_release_note_markdown(&markdown);

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "4px",

            if blocks.is_empty() {
                div {
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    line_height: "1.5",
                    "-"
                }
            } else {
                for (index, block) in blocks.into_iter().enumerate() {
                    {render_release_note_block(block, colors, index)}
                }
            }
        }
    }
}

#[component]
pub fn UpdateDialog(
    update_info: UpdateInfo,
    colors: ThemeColors,
    state: UpdateDialogState,
    progress: (u64, u64),
    error_message: String,
    on_update: EventHandler<()>,
    on_skip: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    let percent = progress_percent(progress.0, progress.1);

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "420px".to_string(),
            title: i18n.read().t("Update available"),

            div {
                div {
                    margin_bottom: "16px",
                    display: "flex",
                    align_items: "center",
                    gap: "12px",

                    div {
                        font_size: "24px",
                        color: "{colors.success}",
                        "🎉"
                    }

                    div {
                        h3 {
                            margin: "0",
                            color: "{colors.text}",
                            font_size: "18px",
                            font_weight: "600",

                            "v{update_info.version}"
                        }

                        if update_info.is_beta {
                            span {
                                margin_left: "8px",
                                padding: "2px 8px",
                                background: "{colors.accent}",
                                color: "{colors.primary_text}",
                                font_size: "11px",
                                border_radius: "4px",

                                "Beta"
                            }
                        }
                    }
                }

                div {
                    margin_bottom: "16px",
                    padding: "12px",
                    background: "{colors.background_tertiary}",
                    border_radius: "6px",
                    max_height: "220px",
                    overflow_y: "auto",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "11px",
                        margin_bottom: "8px",

                        {i18n.read().t("Release notes")}
                    }

                    MarkdownReleaseNotes {
                        markdown: update_info.release_notes.clone(),
                        colors,
                    }
                }

                if state == UpdateDialogState::Downloading {
                    div {
                        margin_bottom: "16px",

                        div {
                            display: "flex",
                            justify_content: "space_between",
                            margin_bottom: "8px",

                            span {
                                color: "{colors.text_secondary}",
                                font_size: "13px",

                                {i18n.read().t("Downloading...")}
                            }

                            span {
                                color: "{colors.accent}",
                                font_size: "13px",

                                "{format_size(progress.0)} / {format_size(progress.1)}"
                            }
                        }

                        div {
                            width: "100%",
                            height: "8px",
                            background: "{colors.background_tertiary}",
                            border_radius: "4px",
                            overflow: "hidden",

                            div {
                                width: "{percent}%",
                                height: "100%",
                                background: "{colors.accent}",
                                border_radius: "4px",
                                transition: "width 0.2s ease",
                            }
                        }
                    }
                }

                if state == UpdateDialogState::Error {
                    div {
                        margin_bottom: "16px",
                        padding: "12px",
                        background: "{colors.error_bg}",
                        border: "1px solid {colors.error}",
                        border_radius: "6px",
                        color: "{colors.error}",
                        font_size: "13px",

                        "{error_message}"
                    }
                }

                if state == UpdateDialogState::Completed {
                    div {
                        margin_bottom: "16px",
                        padding: "12px",
                        background: "{colors.success_bg}",
                        border_radius: "6px",
                        color: "{colors.success}",
                        font_size: "13px",

                        {i18n.read().t("Download complete. Starting installation...")}
                    }
                }

                div {
                    display: "flex",
                    gap: "12px",
                    justify_content: "flex_end",

                    if state == UpdateDialogState::Ready {
                        button {
                            padding: "8px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: {
                                let on_close = on_close.clone();
                                move |_| on_close.call(())
                            },

                            {i18n.read().t("Remind me later")}
                        }

                        button {
                            padding: "8px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text_secondary}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: {
                                let version = update_info.version.clone();
                                let on_skip = on_skip.clone();
                                let on_close = on_close.clone();
                                move |_| {
                                    on_skip.call(version.clone());
                                    on_close.call(())
                                }
                            },

                            {i18n.read().t("Skip this version")}
                        }

                        button {
                            padding: "8px 16px",
                            background: "{colors.primary}",
                            color: "{colors.primary_text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: {
                                let on_update = on_update.clone();
                                move |_| {
                                    on_update.call(())
                                }
                            },

                            {i18n.read().t("Update now")}
                        }
                    } else if state == UpdateDialogState::Downloading {
                        button {
                            padding: "8px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "not_allowed",
                            font_size: "13px",
                            opacity: "0.6",
                            disabled: true,

                            {i18n.read().t("Downloading...")}
                        }
                    } else if state == UpdateDialogState::Error {
                        button {
                            padding: "8px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: {
                                let on_update = on_update.clone();
                                move |_| on_update.call(())
                            },

                            {i18n.read().t("Retry")}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{format_size, parse_release_note_markdown, progress_percent, ReleaseNoteBlock};

    #[test]
    fn parses_markdown_release_notes_into_structured_blocks() {
        let blocks = parse_release_note_markdown(
            "### Added\n- lazy-load key type icons\n- highlight search keyword\n### Fixed\n1. compare prerelease versions",
        );

        assert_eq!(
            blocks,
            vec![
                ReleaseNoteBlock::Heading {
                    level: 3,
                    text: "Added".to_string(),
                },
                ReleaseNoteBlock::Bullet("lazy-load key type icons".to_string()),
                ReleaseNoteBlock::Bullet("highlight search keyword".to_string()),
                ReleaseNoteBlock::Heading {
                    level: 3,
                    text: "Fixed".to_string(),
                },
                ReleaseNoteBlock::Numbered {
                    number: "1.".to_string(),
                    text: "compare prerelease versions".to_string(),
                },
            ]
        );
    }

    #[test]
    fn joins_wrapped_paragraph_lines() {
        let blocks = parse_release_note_markdown("This is a\nwrapped paragraph");

        assert_eq!(
            blocks,
            vec![ReleaseNoteBlock::Paragraph(
                "This is a wrapped paragraph".to_string()
            )]
        );
    }

    #[test]
    fn formats_download_progress_values() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1024), "1 KB");
        assert_eq!(format_size(5 * 1024 * 1024), "5 MB");

        assert_eq!(progress_percent(0, 0), 0);
        assert_eq!(progress_percent(512, 1024), 50);
        assert_eq!(progress_percent(2048, 1024), 100);
    }
}
