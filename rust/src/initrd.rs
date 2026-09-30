static PROGRAMS: &[(&str, &[u8])] = &[
    ("hello", include_bytes!("../../build/hello.elf")),
    ("forktest", include_bytes!("../../build/forktest.elf")),
];

pub fn init() {
    for (name, image) in PROGRAMS {
        let path = alloc::format!("/bin/{}", name);
        match crate::vfs::write_file(&path, image) {
            Ok(()) => crate::logln!(
                "[Initrd] Registered {} ({} bytes).",
                path,
                image.len()
            ),
            Err(err) => crate::logln!("[Initrd] Failed to register {}: {}", path, err),
        }
    }
}
