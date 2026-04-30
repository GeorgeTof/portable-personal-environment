pub const PPE_DEFAULT_CONF: &str = 
r#"lxc.net.0.type = veth
lxc.net.0.link = lxcbr0
lxc.net.0.flags = up

lxc.idmap = u 0 100000 65536
lxc.idmap = g 0 100000 65536

lxc.apparmor.profile = unconfined

# Pass the integrated graphics for hardware acceleration
lxc.mount.entry = /dev/dri dev/dri none bind,optional,create=dir 0 0
"#;