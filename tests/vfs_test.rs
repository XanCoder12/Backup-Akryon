#![allow(dead_code)]

extern crate alloc;

#[path = "../rust/src/vfs.rs"]
mod vfs;

use vfs::InodeType;

#[test]
fn supports_safe_file_and_directory_operations() {
    vfs::init();
    assert_eq!(vfs::current_dir(), "/");

    vfs::make_dir("projects").unwrap();
    vfs::make_dir("projects/src").unwrap();
    assert!(vfs::change_dir("projects/src").is_ok());
    assert_eq!(vfs::current_dir(), "/projects/src");

    vfs::write_file("main.rs", b"fn main() {}\n").unwrap();
    assert_eq!(
        vfs::read_file("./main.rs").as_deref(),
        Some(b"fn main() {}\n".as_slice())
    );
    assert_eq!(vfs::stat("main.rs").unwrap().1, InodeType::File);

    vfs::copy_file("main.rs", "copy.rs").unwrap();
    assert_eq!(vfs::read_file("copy.rs"), vfs::read_file("main.rs"));
    assert!(vfs::copy_file("main.rs", "copy.rs").is_err());

    vfs::move_file("copy.rs", "renamed.rs").unwrap();
    assert!(vfs::read_file("copy.rs").is_none());
    assert!(vfs::read_file("renamed.rs").is_some());

    assert!(vfs::remove_dir("/projects").is_err());
    vfs::remove_file("renamed.rs").unwrap();
    vfs::remove_file("main.rs").unwrap();
    vfs::change_dir("/").unwrap();
    vfs::remove_dir("/projects/src").unwrap();
    vfs::remove_dir("/projects").unwrap();
    assert!(vfs::stat("/projects").is_none());
}
