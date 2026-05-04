pub const PPE_DEFAULT_CONF: &str = 
r#"lxc.net.0.type = veth
lxc.net.0.link = lxcbr0
lxc.net.0.flags = up

lxc.idmap = u 0 100000 65536
lxc.idmap = g 0 100000 65536

lxc.apparmor.profile = unconfined

# pass the integrated graphics for hardware acceleration
lxc.mount.entry = /dev/dri dev/dri none bind,optional,create=dir 0 0
"#;

pub const UNMOUNT_SCRIPT: &str = 
r#"#!/bin/bash
# wait for the Tauri app to completely close
sleep 2

sudo /bin/systemctl stop mnt-ppe.automount
sudo /bin/systemctl stop mnt-ppe.mount

if ! mountpoint -q /mnt/ppe; then
    notify-send -u normal -i drive-removable-media "PPE Unmounted" "It is now safe to remove the external drive."
else
    notify-send -u critical -i drive-removable-media "PPE Error" "Failed to unmount the drive. A background process might still be using it."
fi

# auto-remove to keep /tmp clean
rm -- "$0"
"#;