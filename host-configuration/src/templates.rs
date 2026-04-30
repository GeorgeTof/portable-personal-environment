pub const GPU_UDEV_RULE: &str = 
    "ACTION==\"add|change\", SUBSYSTEM==\"drm\", RUN+=\"/usr/bin/setfacl -m u:100000:rw $env{DEVNAME}\"\n";

pub const IGNORE_PPE_UDEV_RULE: &str = 
    "SUBSYSTEM==\"block\", ENV{ID_FS_LABEL}==\"PPE\", ENV{UDISKS_IGNORE}=\"1\"\n";

pub const MOUNT_UNIT: &str = 
"[Unit]\n\
Description=Mount PPE Drive\n\n\
[Mount]\n\
What=/dev/disk/by-label/PPE\n\
Where=/mnt/ppe\n\
Type=ext4\n\
Options=defaults,noatime,exec\n";

pub const AUTOMOUNT_UNIT: &str = 
"[Unit]\n\
Description=Automount PPE Drive\n\n\
[Automount]\n\
Where=/mnt/ppe\n";

pub const LAUNCHER_STUB_SCRIPT: &str = 
r#"#!/bin/bash

notify_error() {
    notify-send -u critical -i drive-removable-media "PPE Launcher" "$1"
}

sudo /bin/systemctl stop mnt-ppe.automount
sudo /bin/systemctl stop mnt-ppe.mount
sudo /bin/systemctl reset-failed mnt-ppe.automount 2>/dev/null || true
sudo /bin/systemctl reset-failed mnt-ppe.mount 2>/dev/null || true

if [ ! -e "/dev/disk/by-label/PPE" ]; then
    notify_error "PPE drive not found! Please insert the external drive and try again."
    exit 1
fi

sudo /bin/systemctl start mnt-ppe.automount

#SESSION_SCRIPT="/mnt/ppe/session-launcher_0.1.0_amd64.AppImage"
SESSION_SCRIPT="/mnt/ppe/session-launcher"

if [ ! -f "$SESSION_SCRIPT" ]; then
    notify_error "Session script (session-launcher.AppImage) not found on the drive!"
    sudo /bin/systemctl stop mnt-ppe.automount
    exit 1
fi

exec "$SESSION_SCRIPT"
"#;

pub const DESKTOP_ENTRY: &str = 
"[Desktop Entry]\n\
Name=PPE Launcher\n\
Comment=Launch the Portable Personal Environment\n\
Exec=/usr/local/bin/ppe-launcher-stub\n\
Icon=drive-removable-media\n\
Terminal=false\n\
Type=Application\n\
Categories=Utility;System;\n";