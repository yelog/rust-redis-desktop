use dioxus::prelude::*;
use dioxus_free_icons::{icons::ld_icons::*, Icon};

macro_rules! define_icon {
    ($name:ident, $icon:ident) => {
        #[component]
        pub fn $name(size: Option<i32>, color: Option<String>) -> Element {
            let size = size.unwrap_or(16).max(1) as u32;
            let color = color.unwrap_or_else(|| "currentColor".to_string());

            rsx! {
                Icon {
                    icon: $icon,
                    width: Some(size),
                    height: Some(size),
                    fill: color,
                }
            }
        }
    };
}

define_icon!(IconCopy, LdCopy);
define_icon!(IconEdit, LdPencil);
define_icon!(IconTrash, LdTrash2);
define_icon!(IconFile, LdFileText);
define_icon!(IconFolder, LdFolder);
define_icon!(IconList, LdList);
define_icon!(IconCheck, LdCheck);
define_icon!(IconSettings, LdSettings);
define_icon!(IconRefresh, LdRefreshCw);
define_icon!(IconX, LdX);
define_icon!(IconAlert, LdTriangleAlert);
define_icon!(IconDatabase, LdDatabase);
define_icon!(IconHash, LdBraces);
define_icon!(IconSet, LdCircleDot);
define_icon!(IconZSet, LdBarChart);
define_icon!(IconStream, LdWaves);
define_icon!(IconPlus, LdPlus);
define_icon!(IconSearch, LdSearch);
define_icon!(IconKey, LdKey);
define_icon!(IconBell, LdBell);
define_icon!(IconHelpCircle, LdCircleHelp);
define_icon!(IconTerminal, LdTerminal);
define_icon!(IconActivity, LdActivity);
define_icon!(IconUsers, LdUsers);
define_icon!(IconClock, LdClock);
define_icon!(IconDownload, LdDownload);
define_icon!(IconUpload, LdUpload);
define_icon!(IconStar, LdStar);
define_icon!(IconSquare, LdSquare);
define_icon!(IconExternalLink, LdExternalLink);
define_icon!(IconGlobe, LdGlobe);
define_icon!(IconGitHub, LdGithub);
define_icon!(IconMoreHorizontal, LdEllipsis);
define_icon!(IconChevronRight, LdChevronRight);
define_icon!(IconChevronDown, LdChevronDown);
define_icon!(IconJson, LdFileJson);
