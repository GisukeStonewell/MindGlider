//! Simplified Freeplane resource images. See assets/freeplane-icons/README.md.
pub struct Icon {
    pub id: &'static str,
    pub key: &'static str,
    pub image: &'static str,
}
pub static CATALOG: &[Icon] = &[
    Icon {
        id: "help",
        key: "icon.help",
        image: include_str!("../assets/freeplane-icons/simplified/help.svg"),
    },
    Icon {
        id: "yes",
        key: "icon.yes",
        image: include_str!("../assets/freeplane-icons/simplified/yes.svg"),
    },
    Icon {
        id: "button_ok",
        key: "icon.button_ok",
        image: include_str!("../assets/freeplane-icons/simplified/button_ok.svg"),
    },
    Icon {
        id: "button_cancel",
        key: "icon.button_cancel",
        image: include_str!("../assets/freeplane-icons/simplified/button_cancel.svg"),
    },
    Icon {
        id: "bookmark",
        key: "icon.bookmark",
        image: include_str!("../assets/freeplane-icons/simplified/bookmark.svg"),
    },
    Icon {
        id: "idea",
        key: "icon.idea",
        image: include_str!("../assets/freeplane-icons/simplified/idea.svg"),
    },
    Icon {
        id: "messagebox_warning",
        key: "icon.messagebox_warning",
        image: include_str!("../assets/freeplane-icons/simplified/messagebox_warning.svg"),
    },
    Icon {
        id: "stop-sign",
        key: "icon.stop-sign",
        image: include_str!("../assets/freeplane-icons/simplified/stop-sign.svg"),
    },
    Icon {
        id: "closed",
        key: "icon.closed",
        image: include_str!("../assets/freeplane-icons/simplified/closed.svg"),
    },
    Icon {
        id: "info",
        key: "icon.info",
        image: include_str!("../assets/freeplane-icons/simplified/info.svg"),
    },
    Icon {
        id: "clanbomber",
        key: "icon.clanbomber",
        image: include_str!("../assets/freeplane-icons/simplified/clanbomber.svg"),
    },
    Icon {
        id: "checked",
        key: "icon.checked",
        image: include_str!("../assets/freeplane-icons/simplified/checked.svg"),
    },
    Icon {
        id: "unchecked",
        key: "icon.unchecked",
        image: include_str!("../assets/freeplane-icons/simplified/unchecked.svg"),
    },
    Icon {
        id: "wizard",
        key: "icon.wizard",
        image: include_str!("../assets/freeplane-icons/simplified/wizard.svg"),
    },
    Icon {
        id: "gohome",
        key: "icon.gohome",
        image: include_str!("../assets/freeplane-icons/simplified/gohome.svg"),
    },
    Icon {
        id: "knotify",
        key: "icon.knotify",
        image: include_str!("../assets/freeplane-icons/simplified/knotify.svg"),
    },
    Icon {
        id: "password",
        key: "icon.password",
        image: include_str!("../assets/freeplane-icons/simplified/password.svg"),
    },
    Icon {
        id: "pencil",
        key: "icon.pencil",
        image: include_str!("../assets/freeplane-icons/simplified/pencil.svg"),
    },
    Icon {
        id: "xmag",
        key: "icon.xmag",
        image: include_str!("../assets/freeplane-icons/simplified/xmag.svg"),
    },
    Icon {
        id: "bell",
        key: "icon.bell",
        image: include_str!("../assets/freeplane-icons/simplified/bell.svg"),
    },
    Icon {
        id: "launch",
        key: "icon.launch",
        image: include_str!("../assets/freeplane-icons/simplified/launch.svg"),
    },
    Icon {
        id: "broken-line",
        key: "icon.broken-line",
        image: include_str!("../assets/freeplane-icons/simplified/broken-line.svg"),
    },
    Icon {
        id: "stop",
        key: "icon.stop",
        image: include_str!("../assets/freeplane-icons/simplified/stop.svg"),
    },
    Icon {
        id: "prepare",
        key: "icon.prepare",
        image: include_str!("../assets/freeplane-icons/simplified/prepare.svg"),
    },
    Icon {
        id: "go",
        key: "icon.go",
        image: include_str!("../assets/freeplane-icons/simplified/go.svg"),
    },
    Icon {
        id: "very_negative",
        key: "icon.very_negative",
        image: include_str!("../assets/freeplane-icons/simplified/very_negative.svg"),
    },
    Icon {
        id: "negative",
        key: "icon.negative",
        image: include_str!("../assets/freeplane-icons/simplified/negative.svg"),
    },
    Icon {
        id: "neutral",
        key: "icon.neutral",
        image: include_str!("../assets/freeplane-icons/simplified/neutral.svg"),
    },
    Icon {
        id: "positive",
        key: "icon.positive",
        image: include_str!("../assets/freeplane-icons/simplified/positive.svg"),
    },
    Icon {
        id: "very_positive",
        key: "icon.very_positive",
        image: include_str!("../assets/freeplane-icons/simplified/very_positive.svg"),
    },
    Icon {
        id: "full-1",
        key: "icon.full-1",
        image: include_str!("../assets/freeplane-icons/simplified/full-1.svg"),
    },
    Icon {
        id: "full-2",
        key: "icon.full-2",
        image: include_str!("../assets/freeplane-icons/simplified/full-2.svg"),
    },
    Icon {
        id: "full-3",
        key: "icon.full-3",
        image: include_str!("../assets/freeplane-icons/simplified/full-3.svg"),
    },
    Icon {
        id: "full-4",
        key: "icon.full-4",
        image: include_str!("../assets/freeplane-icons/simplified/full-4.svg"),
    },
    Icon {
        id: "full-5",
        key: "icon.full-5",
        image: include_str!("../assets/freeplane-icons/simplified/full-5.svg"),
    },
    Icon {
        id: "full-6",
        key: "icon.full-6",
        image: include_str!("../assets/freeplane-icons/simplified/full-6.svg"),
    },
    Icon {
        id: "full-7",
        key: "icon.full-7",
        image: include_str!("../assets/freeplane-icons/simplified/full-7.svg"),
    },
    Icon {
        id: "full-8",
        key: "icon.full-8",
        image: include_str!("../assets/freeplane-icons/simplified/full-8.svg"),
    },
    Icon {
        id: "full-9",
        key: "icon.full-9",
        image: include_str!("../assets/freeplane-icons/simplified/full-9.svg"),
    },
    Icon {
        id: "full-0",
        key: "icon.full-0",
        image: include_str!("../assets/freeplane-icons/simplified/full-0.svg"),
    },
    Icon {
        id: "0%",
        key: "icon.0%",
        image: include_str!("../assets/freeplane-icons/simplified/0%.svg"),
    },
    Icon {
        id: "25%",
        key: "icon.25%",
        image: include_str!("../assets/freeplane-icons/simplified/25%.svg"),
    },
    Icon {
        id: "50%",
        key: "icon.50%",
        image: include_str!("../assets/freeplane-icons/simplified/50%.svg"),
    },
    Icon {
        id: "75%",
        key: "icon.75%",
        image: include_str!("../assets/freeplane-icons/simplified/75%.svg"),
    },
    Icon {
        id: "100%",
        key: "icon.100%",
        image: include_str!("../assets/freeplane-icons/simplified/100%.svg"),
    },
    Icon {
        id: "ksmiletris",
        key: "icon.ksmiletris",
        image: include_str!("../assets/freeplane-icons/simplified/ksmiletris.svg"),
    },
    Icon {
        id: "smiley-neutral",
        key: "icon.smiley-neutral",
        image: include_str!("../assets/freeplane-icons/simplified/smiley-neutral.svg"),
    },
    Icon {
        id: "smiley-oh",
        key: "icon.smiley-oh",
        image: include_str!("../assets/freeplane-icons/simplified/smiley-oh.svg"),
    },
    Icon {
        id: "smiley-angry",
        key: "icon.smiley-angry",
        image: include_str!("../assets/freeplane-icons/simplified/smiley-angry.svg"),
    },
    Icon {
        id: "smily_bad",
        key: "icon.smily_bad",
        image: include_str!("../assets/freeplane-icons/simplified/smily_bad.svg"),
    },
    Icon {
        id: "family",
        key: "icon.family",
        image: include_str!("../assets/freeplane-icons/simplified/family.svg"),
    },
    Icon {
        id: "female1",
        key: "icon.female1",
        image: include_str!("../assets/freeplane-icons/simplified/female1.svg"),
    },
    Icon {
        id: "female2",
        key: "icon.female2",
        image: include_str!("../assets/freeplane-icons/simplified/female2.svg"),
    },
    Icon {
        id: "females",
        key: "icon.females",
        image: include_str!("../assets/freeplane-icons/simplified/females.svg"),
    },
    Icon {
        id: "male1",
        key: "icon.male1",
        image: include_str!("../assets/freeplane-icons/simplified/male1.svg"),
    },
    Icon {
        id: "male2",
        key: "icon.male2",
        image: include_str!("../assets/freeplane-icons/simplified/male2.svg"),
    },
    Icon {
        id: "males",
        key: "icon.males",
        image: include_str!("../assets/freeplane-icons/simplified/males.svg"),
    },
    Icon {
        id: "fema",
        key: "icon.fema",
        image: include_str!("../assets/freeplane-icons/simplified/fema.svg"),
    },
    Icon {
        id: "group",
        key: "icon.group",
        image: include_str!("../assets/freeplane-icons/simplified/group.svg"),
    },
    Icon {
        id: "licq",
        key: "icon.licq",
        image: include_str!("../assets/freeplane-icons/simplified/licq.svg"),
    },
    Icon {
        id: "penguin",
        key: "icon.penguin",
        image: include_str!("../assets/freeplane-icons/simplified/penguin.svg"),
    },
    Icon {
        id: "freemind_butterfly",
        key: "icon.freemind_butterfly",
        image: include_str!("../assets/freeplane-icons/simplified/freemind_butterfly.svg"),
    },
    Icon {
        id: "bee",
        key: "icon.bee",
        image: include_str!("../assets/freeplane-icons/simplified/bee.svg"),
    },
    Icon {
        id: "clock",
        key: "icon.clock",
        image: include_str!("../assets/freeplane-icons/simplified/clock.svg"),
    },
    Icon {
        id: "clock2",
        key: "icon.clock2",
        image: include_str!("../assets/freeplane-icons/simplified/clock2.svg"),
    },
    Icon {
        id: "hourglass",
        key: "icon.hourglass",
        image: include_str!("../assets/freeplane-icons/simplified/hourglass.svg"),
    },
    Icon {
        id: "calendar",
        key: "icon.calendar",
        image: include_str!("../assets/freeplane-icons/simplified/calendar.svg"),
    },
    Icon {
        id: "attach",
        key: "icon.attach",
        image: include_str!("../assets/freeplane-icons/simplified/attach.svg"),
    },
    Icon {
        id: "desktop_new",
        key: "icon.desktop_new",
        image: include_str!("../assets/freeplane-icons/simplified/desktop_new.svg"),
    },
    Icon {
        id: "list",
        key: "icon.list",
        image: include_str!("../assets/freeplane-icons/simplified/list.svg"),
    },
    Icon {
        id: "edit",
        key: "icon.edit",
        image: include_str!("../assets/freeplane-icons/simplified/edit.svg"),
    },
    Icon {
        id: "kaddressbook",
        key: "icon.kaddressbook",
        image: include_str!("../assets/freeplane-icons/simplified/kaddressbook.svg"),
    },
    Icon {
        id: "folder",
        key: "icon.folder",
        image: include_str!("../assets/freeplane-icons/simplified/folder.svg"),
    },
    Icon {
        id: "kmail",
        key: "icon.kmail",
        image: include_str!("../assets/freeplane-icons/simplified/kmail.svg"),
    },
    Icon {
        id: "Mail",
        key: "icon.Mail",
        image: include_str!("../assets/freeplane-icons/simplified/Mail.svg"),
    },
    Icon {
        id: "revision",
        key: "icon.revision",
        image: include_str!("../assets/freeplane-icons/simplified/revision.svg"),
    },
    Icon {
        id: "revision-green",
        key: "icon.revision-green",
        image: include_str!("../assets/freeplane-icons/simplified/revision-green.svg"),
    },
    Icon {
        id: "revision-pink",
        key: "icon.revision-pink",
        image: include_str!("../assets/freeplane-icons/simplified/revision-pink.svg"),
    },
    Icon {
        id: "revision-red",
        key: "icon.revision-red",
        image: include_str!("../assets/freeplane-icons/simplified/revision-red.svg"),
    },
    Icon {
        id: "video",
        key: "icon.video",
        image: include_str!("../assets/freeplane-icons/simplified/video.svg"),
    },
    Icon {
        id: "audio",
        key: "icon.audio",
        image: include_str!("../assets/freeplane-icons/simplified/audio.svg"),
    },
    Icon {
        id: "executable",
        key: "icon.executable",
        image: include_str!("../assets/freeplane-icons/simplified/executable.svg"),
    },
    Icon {
        id: "image",
        key: "icon.image",
        image: include_str!("../assets/freeplane-icons/simplified/image.svg"),
    },
    Icon {
        id: "internet",
        key: "icon.internet",
        image: include_str!("../assets/freeplane-icons/simplified/internet.svg"),
    },
    Icon {
        id: "internet_warning",
        key: "icon.internet_warning",
        image: include_str!("../assets/freeplane-icons/simplified/internet_warning.svg"),
    },
    Icon {
        id: "mindmap",
        key: "icon.mindmap",
        image: include_str!("../assets/freeplane-icons/simplified/mindmap.svg"),
    },
    Icon {
        id: "narrative",
        key: "icon.narrative",
        image: include_str!("../assets/freeplane-icons/simplified/narrative.svg"),
    },
    Icon {
        id: "flag-black",
        key: "icon.flag-black",
        image: include_str!("../assets/freeplane-icons/simplified/flag-black.svg"),
    },
    Icon {
        id: "flag-blue",
        key: "icon.flag-blue",
        image: include_str!("../assets/freeplane-icons/simplified/flag-blue.svg"),
    },
    Icon {
        id: "flag-green",
        key: "icon.flag-green",
        image: include_str!("../assets/freeplane-icons/simplified/flag-green.svg"),
    },
    Icon {
        id: "flag-orange",
        key: "icon.flag-orange",
        image: include_str!("../assets/freeplane-icons/simplified/flag-orange.svg"),
    },
    Icon {
        id: "flag-pink",
        key: "icon.flag-pink",
        image: include_str!("../assets/freeplane-icons/simplified/flag-pink.svg"),
    },
    Icon {
        id: "flag",
        key: "icon.flag",
        image: include_str!("../assets/freeplane-icons/simplified/flag.svg"),
    },
    Icon {
        id: "flag-yellow",
        key: "icon.flag-yellow",
        image: include_str!("../assets/freeplane-icons/simplified/flag-yellow.svg"),
    },
    Icon {
        id: "forward",
        key: "icon.forward",
        image: include_str!("../assets/freeplane-icons/simplified/forward.svg"),
    },
    Icon {
        id: "back",
        key: "icon.back",
        image: include_str!("../assets/freeplane-icons/simplified/back.svg"),
    },
    Icon {
        id: "up",
        key: "icon.up",
        image: include_str!("../assets/freeplane-icons/simplified/up.svg"),
    },
    Icon {
        id: "down",
        key: "icon.down",
        image: include_str!("../assets/freeplane-icons/simplified/down.svg"),
    },
    Icon {
        id: "addition",
        key: "icon.addition",
        image: include_str!("../assets/freeplane-icons/simplified/addition.svg"),
    },
    Icon {
        id: "subtraction",
        key: "icon.subtraction",
        image: include_str!("../assets/freeplane-icons/simplified/subtraction.svg"),
    },
    Icon {
        id: "multiplication",
        key: "icon.multiplication",
        image: include_str!("../assets/freeplane-icons/simplified/multiplication.svg"),
    },
    Icon {
        id: "division",
        key: "icon.division",
        image: include_str!("../assets/freeplane-icons/simplified/division.svg"),
    },
    Icon {
        id: "links/DecoratedLink",
        key: "icon.links/DecoratedLink",
        image: include_str!("../assets/freeplane-icons/simplified/links/DecoratedLink.svg"),
    },
    Icon {
        id: "links/DecoratedLinkLocal",
        key: "icon.links/DecoratedLinkLocal",
        image: include_str!("../assets/freeplane-icons/simplified/links/DecoratedLinkLocal.svg"),
    },
    Icon {
        id: "links/DecoratedMail",
        key: "icon.links/DecoratedMail",
        image: include_str!("../assets/freeplane-icons/simplified/links/DecoratedMail.svg"),
    },
    Icon {
        id: "links/microsoft/file_ms_project",
        key: "icon.links/microsoft/file_ms_project",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/microsoft/file_ms_project.svg"
        ),
    },
    Icon {
        id: "links/microsoft/file_ms_word",
        key: "icon.links/microsoft/file_ms_word",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/microsoft/file_ms_word.svg"
        ),
    },
    Icon {
        id: "links/microsoft/file_ms_onenote",
        key: "icon.links/microsoft/file_ms_onenote",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/microsoft/file_ms_onenote.svg"
        ),
    },
    Icon {
        id: "links/microsoft/file_ms_outlook",
        key: "icon.links/microsoft/file_ms_outlook",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/microsoft/file_ms_outlook.svg"
        ),
    },
    Icon {
        id: "links/microsoft/file_ms_access",
        key: "icon.links/microsoft/file_ms_access",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/microsoft/file_ms_access.svg"
        ),
    },
    Icon {
        id: "links/microsoft/file_ms_powerpoint",
        key: "icon.links/microsoft/file_ms_powerpoint",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/microsoft/file_ms_powerpoint.svg"
        ),
    },
    Icon {
        id: "links/microsoft/file_ms_visio",
        key: "icon.links/microsoft/file_ms_visio",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/microsoft/file_ms_visio.svg"
        ),
    },
    Icon {
        id: "links/microsoft/file_ms_excel",
        key: "icon.links/microsoft/file_ms_excel",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/microsoft/file_ms_excel.svg"
        ),
    },
    Icon {
        id: "links/code/file_SCSS",
        key: "icon.links/code/file_SCSS",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_SCSS.svg"),
    },
    Icon {
        id: "links/code/file_XSD",
        key: "icon.links/code/file_XSD",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_XSD.svg"),
    },
    Icon {
        id: "links/code/file_Xcode_project",
        key: "icon.links/code/file_Xcode_project",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/code/file_Xcode_project.svg"
        ),
    },
    Icon {
        id: "links/code/file_Javascript",
        key: "icon.links/code/file_Javascript",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_Javascript.svg"),
    },
    Icon {
        id: "links/code/file_Typescript",
        key: "icon.links/code/file_Typescript",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_Typescript.svg"),
    },
    Icon {
        id: "links/code/file_Python",
        key: "icon.links/code/file_Python",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_Python.svg"),
    },
    Icon {
        id: "links/code/file_Swift",
        key: "icon.links/code/file_Swift",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_Swift.svg"),
    },
    Icon {
        id: "links/code/file_Java",
        key: "icon.links/code/file_Java",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_Java.svg"),
    },
    Icon {
        id: "links/code/file_Xcode_playground",
        key: "icon.links/code/file_Xcode_playground",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/code/file_Xcode_playground.svg"
        ),
    },
    Icon {
        id: "links/code/file_AppleScript",
        key: "icon.links/code/file_AppleScript",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_AppleScript.svg"),
    },
    Icon {
        id: "links/code/file_C++_cpp",
        key: "icon.links/code/file_C++_cpp",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_C++_cpp.svg"),
    },
    Icon {
        id: "links/code/file_XML",
        key: "icon.links/code/file_XML",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_XML.svg"),
    },
    Icon {
        id: "links/sysadmin/shell_script",
        key: "icon.links/sysadmin/shell_script",
        image: include_str!("../assets/freeplane-icons/simplified/links/sysadmin/shell_script.svg"),
    },
    Icon {
        id: "links/app/MarginNote",
        key: "icon.links/app/MarginNote",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/MarginNote.svg"),
    },
    Icon {
        id: "links/app/Git",
        key: "icon.links/app/Git",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/Git.svg"),
    },
    Icon {
        id: "links/app/Scapple",
        key: "icon.links/app/Scapple",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/Scapple.svg"),
    },
    Icon {
        id: "links/app/Things",
        key: "icon.links/app/Things",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/Things.svg"),
    },
    Icon {
        id: "links/app/Accordance",
        key: "icon.links/app/Accordance",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/Accordance.svg"),
    },
    Icon {
        id: "links/app/QuickTime",
        key: "icon.links/app/QuickTime",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/QuickTime.svg"),
    },
    Icon {
        id: "links/app/Marked2",
        key: "icon.links/app/Marked2",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/Marked2.svg"),
    },
    Icon {
        id: "links/app/Eclipse",
        key: "icon.links/app/Eclipse",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/Eclipse.svg"),
    },
    Icon {
        id: "links/app/OmniGraffle",
        key: "icon.links/app/OmniGraffle",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/OmniGraffle.svg"),
    },
    Icon {
        id: "links/app/Bear",
        key: "icon.links/app/Bear",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/Bear.svg"),
    },
    Icon {
        id: "links/design/file_affinity_designer",
        key: "icon.links/design/file_affinity_designer",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/design/file_affinity_designer.svg"
        ),
    },
    Icon {
        id: "links/design/file_affinity_photo",
        key: "icon.links/design/file_affinity_photo",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/design/file_affinity_photo.svg"
        ),
    },
    Icon {
        id: "links/design/file_adobe_indesign",
        key: "icon.links/design/file_adobe_indesign",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/design/file_adobe_indesign.svg"
        ),
    },
    Icon {
        id: "links/design/file_sketch",
        key: "icon.links/design/file_sketch",
        image: include_str!("../assets/freeplane-icons/simplified/links/design/file_sketch.svg"),
    },
    Icon {
        id: "links/design/file_affinity_publisher",
        key: "icon.links/design/file_affinity_publisher",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/design/file_affinity_publisher.svg"
        ),
    },
    Icon {
        id: "links/folder/folder",
        key: "icon.links/folder/folder",
        image: include_str!("../assets/freeplane-icons/simplified/links/folder/folder.svg"),
    },
    Icon {
        id: "links/link/network_share",
        key: "icon.links/link/network_share",
        image: include_str!("../assets/freeplane-icons/simplified/links/link/network_share.svg"),
    },
    Icon {
        id: "links/link/web_ssl",
        key: "icon.links/link/web_ssl",
        image: include_str!("../assets/freeplane-icons/simplified/links/link/web_ssl.svg"),
    },
    Icon {
        id: "links/link/web",
        key: "icon.links/link/web",
        image: include_str!("../assets/freeplane-icons/simplified/links/link/web.svg"),
    },
    Icon {
        id: "links/file/png",
        key: "icon.links/file/png",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/png.svg"),
    },
    Icon {
        id: "links/file/webarchive",
        key: "icon.links/file/webarchive",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/webarchive.svg"),
    },
    Icon {
        id: "links/file/markdown",
        key: "icon.links/file/markdown",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/markdown.svg"),
    },
    Icon {
        id: "links/file/video_avi",
        key: "icon.links/file/video_avi",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/video_avi.svg"),
    },
    Icon {
        id: "links/file/cmd",
        key: "icon.links/file/cmd",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/cmd.svg"),
    },
    Icon {
        id: "links/file/batch",
        key: "icon.links/file/batch",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/batch.svg"),
    },
    Icon {
        id: "links/file/txt",
        key: "icon.links/file/txt",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/txt.svg"),
    },
    Icon {
        id: "links/file/html",
        key: "icon.links/file/html",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/html.svg"),
    },
    Icon {
        id: "links/file/freeplane_mindmap",
        key: "icon.links/file/freeplane_mindmap",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/file/freeplane_mindmap.svg"
        ),
    },
    Icon {
        id: "links/file/pdf",
        key: "icon.links/file/pdf",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/pdf.svg"),
    },
    Icon {
        id: "links/file/email_envelope",
        key: "icon.links/file/email_envelope",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/email_envelope.svg"),
    },
    Icon {
        id: "links/file/png_image_graphic",
        key: "icon.links/file/png_image_graphic",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/file/png_image_graphic.svg"
        ),
    },
    Icon {
        id: "links/file/email",
        key: "icon.links/file/email",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/email.svg"),
    },
    Icon {
        id: "links/file/json",
        key: "icon.links/file/json",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/json.svg"),
    },
    Icon {
        id: "links/file/book_ePub",
        key: "icon.links/file/book_ePub",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/book_ePub.svg"),
    },
    Icon {
        id: "links/file/exe",
        key: "icon.links/file/exe",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/exe.svg"),
    },
    Icon {
        id: "links/file/tsv",
        key: "icon.links/file/tsv",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/tsv.svg"),
    },
    Icon {
        id: "links/file/zip",
        key: "icon.links/file/zip",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/zip.svg"),
    },
    Icon {
        id: "links/file/m4a_audio_sound_music",
        key: "icon.links/file/m4a_audio_sound_music",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/file/m4a_audio_sound_music.svg"
        ),
    },
    Icon {
        id: "links/file/audio_mp3_music",
        key: "icon.links/file/audio_mp3_music",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/audio_mp3_music.svg"),
    },
    Icon {
        id: "links/file/generic",
        key: "icon.links/file/generic",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/generic.svg"),
    },
    Icon {
        id: "links/file/css",
        key: "icon.links/file/css",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/css.svg"),
    },
    Icon {
        id: "links/file/csv",
        key: "icon.links/file/csv",
        image: include_str!("../assets/freeplane-icons/simplified/links/file/csv.svg"),
    },
    Icon {
        id: "links/libreoffice/file_doc_texthtml",
        key: "icon.links/libreoffice/file_doc_texthtml",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_doc_texthtml.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_doc_presentation",
        key: "icon.links/libreoffice/file_doc_presentation",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_doc_presentation.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_template_drawing",
        key: "icon.links/libreoffice/file_template_drawing",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_template_drawing.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_template_spreadsheet",
        key: "icon.links/libreoffice/file_template_spreadsheet",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_template_spreadsheet.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_doc_spreadsheet",
        key: "icon.links/libreoffice/file_doc_spreadsheet",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_doc_spreadsheet.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_template_texthtml",
        key: "icon.links/libreoffice/file_template_texthtml",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_template_texthtml.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_template_presentation",
        key: "icon.links/libreoffice/file_template_presentation",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_template_presentation.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_doc_drawing",
        key: "icon.links/libreoffice/file_doc_drawing",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_doc_drawing.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_template_text",
        key: "icon.links/libreoffice/file_template_text",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_template_text.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_doc_text",
        key: "icon.links/libreoffice/file_doc_text",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_doc_text.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/file_doc_database",
        key: "icon.links/libreoffice/file_doc_database",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/file_doc_database.svg"
        ),
    },
    Icon {
        id: "decrypted",
        key: "icon.decrypted",
        image: include_str!("../assets/freeplane-icons/simplified/decrypted.svg"),
    },
    Icon {
        id: "encrypted",
        key: "icon.encrypted",
        image: include_str!("../assets/freeplane-icons/simplified/encrypted.svg"),
    },
    Icon {
        id: "korn",
        key: "icon.korn",
        image: include_str!("../assets/freeplane-icons/simplified/korn.svg"),
    },
    Icon {
        id: "links/app/QuickTime_alt",
        key: "icon.links/app/QuickTime_alt",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/QuickTime_alt.svg"),
    },
    Icon {
        id: "links/app/Sketch",
        key: "icon.links/app/Sketch",
        image: include_str!("../assets/freeplane-icons/simplified/links/app/Sketch.svg"),
    },
    Icon {
        id: "links/cloud/cloud_AWS",
        key: "icon.links/cloud/cloud_AWS",
        image: include_str!("../assets/freeplane-icons/simplified/links/cloud/cloud_AWS.svg"),
    },
    Icon {
        id: "links/cloud/cloud_Azure",
        key: "icon.links/cloud/cloud_Azure",
        image: include_str!("../assets/freeplane-icons/simplified/links/cloud/cloud_Azure.svg"),
    },
    Icon {
        id: "links/code/file_CSS",
        key: "icon.links/code/file_CSS",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_CSS.svg"),
    },
    Icon {
        id: "links/code/file_RxSwift",
        key: "icon.links/code/file_RxSwift",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_RxSwift.svg"),
    },
    Icon {
        id: "links/code/file_Shell",
        key: "icon.links/code/file_Shell",
        image: include_str!("../assets/freeplane-icons/simplified/links/code/file_Shell.svg"),
    },
    Icon {
        id: "links/design/file_photoshop_psd",
        key: "icon.links/design/file_photoshop_psd",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/design/file_photoshop_psd.svg"
        ),
    },
    Icon {
        id: "links/folder/inputs",
        key: "icon.links/folder/inputs",
        image: include_str!("../assets/freeplane-icons/simplified/links/folder/inputs.svg"),
    },
    Icon {
        id: "links/folder/key",
        key: "icon.links/folder/key",
        image: include_str!("../assets/freeplane-icons/simplified/links/folder/key.svg"),
    },
    Icon {
        id: "links/folder/outputs",
        key: "icon.links/folder/outputs",
        image: include_str!("../assets/freeplane-icons/simplified/links/folder/outputs.svg"),
    },
    Icon {
        id: "links/folder/task",
        key: "icon.links/folder/task",
        image: include_str!("../assets/freeplane-icons/simplified/links/folder/task.svg"),
    },
    Icon {
        id: "links/folder/user",
        key: "icon.links/folder/user",
        image: include_str!("../assets/freeplane-icons/simplified/links/folder/user.svg"),
    },
    Icon {
        id: "links/libreoffice/chart",
        key: "icon.links/libreoffice/chart",
        image: include_str!("../assets/freeplane-icons/simplified/links/libreoffice/chart.svg"),
    },
    Icon {
        id: "links/libreoffice/installer",
        key: "icon.links/libreoffice/installer",
        image: include_str!("../assets/freeplane-icons/simplified/links/libreoffice/installer.svg"),
    },
    Icon {
        id: "links/libreoffice/macro",
        key: "icon.links/libreoffice/macro",
        image: include_str!("../assets/freeplane-icons/simplified/links/libreoffice/macro.svg"),
    },
    Icon {
        id: "links/libreoffice/master_document",
        key: "icon.links/libreoffice/master_document",
        image: include_str!(
            "../assets/freeplane-icons/simplified/links/libreoffice/master_document.svg"
        ),
    },
    Icon {
        id: "links/libreoffice/math",
        key: "icon.links/libreoffice/math",
        image: include_str!("../assets/freeplane-icons/simplified/links/libreoffice/math.svg"),
    },
    Icon {
        id: "user_icon",
        key: "icon.user_icon",
        image: include_str!("../assets/freeplane-icons/simplified/user_icon.svg"),
    },
];
pub fn find(name: &str) -> Option<&'static Icon> {
    CATALOG.iter().find(|i| i.id == name)
}
pub fn body(name: &str) -> String {
    if let Some(icon) = find(name) {
        return icon.image.into();
    }
    if let Some(progress) = name
        .strip_suffix('%')
        .and_then(|p| p.parse::<u8>().ok())
        .filter(|p| *p <= 100)
    {
        return format!(
            "<g fill=\"none\" stroke=\"#506780\"><circle cx=\"12\" cy=\"12\" r=\"9\" stroke-width=\"2\" stroke-opacity=\".25\"/><circle cx=\"12\" cy=\"12\" r=\"9\" stroke-width=\"4\" stroke-dasharray=\"{} 56.55\" transform=\"rotate(-90 12 12)\"/></g>",
            56.55 * progress as f32 / 100.
        );
    }
    find("help").unwrap().image.into()
}
pub fn svg(name: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\">{}</svg>",
        body(name)
    )
}
/// Toggle a palette icon atomically through the caller's History transaction.
pub fn toggle(
    doc: &mut crate::model::Document,
    targets: &std::collections::BTreeSet<String>,
    id: &str,
) -> anyhow::Result<()> {
    anyhow::ensure!(find(id).is_some(), "Unknown built-in icon ID");
    for target in targets {
        doc.ensure_editable(target)?;
    }
    for target in targets {
        let node = doc.nodes.get_mut(target).unwrap();
        if let Some(p) = id.strip_suffix('%').and_then(|s| s.parse::<u8>().ok()) {
            node.progress = if node.progress == Some(p) {
                None
            } else {
                Some(p)
            };
        } else if node.icons.iter().any(|i| i == id) {
            node.icons.retain(|i| i != id);
        } else {
            node.icons.push(id.into());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn every_imported_icon_is_unique_and_renderable() {
        let mut ids = std::collections::BTreeSet::new();
        for icon in super::CATALOG {
            assert!(ids.insert(icon.id));
            let tree =
                resvg::usvg::Tree::from_str(&super::svg(icon.id), &Default::default()).unwrap();
            assert!(!tree.root().children().is_empty(), "{}", icon.id);
        }
    }
}
