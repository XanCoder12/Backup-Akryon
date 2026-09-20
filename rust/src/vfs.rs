use alloc::string::String;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    File,
    Directory,
    CharDevice,
}

pub trait InodeOperations {
    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize, i32>;
    fn write(&mut self, offset: usize, buf: &[u8]) -> Result<usize, i32>;
    fn get_size(&self) -> usize;
}

pub struct MemoryInode {
    pub name: String,
    pub inode_type: InodeType,
    pub data: Vec<u8>,
}

impl InodeOperations for MemoryInode {
    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize, i32> {
        if offset >= self.data.len() {
            return Ok(0);
        }
        let available = self.data.len() - offset;
        let to_read = available.min(buf.len());
        buf[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
        Ok(to_read)
    }

    fn write(&mut self, offset: usize, buf: &[u8]) -> Result<usize, i32> {
        let needed_len = offset + buf.len();
        if needed_len > self.data.len() {
            self.data.resize(needed_len, 0);
        }
        self.data[offset..needed_len].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn get_size(&self) -> usize {
        self.data.len()
    }
}

pub struct RamFs {
    files: Vec<MemoryInode>,
    current_dir: String,
}

struct SafeRamFs(UnsafeCell<Option<RamFs>>);
unsafe impl Sync for SafeRamFs {}

static RAMFS: SafeRamFs = SafeRamFs(UnsafeCell::new(None));

fn normalize_path(cwd: &str, input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    let base = if input.starts_with('/') { "/" } else { cwd };
    let mut parts: Vec<&str> = Vec::new();
    for part in base.split('/').chain(input.split('/')) {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            value => parts.push(value),
        }
    }

    let mut path = String::from("/");
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            path.push('/');
        }
        path.push_str(part);
    }
    Some(path)
}

fn parent_path(path: &str) -> &str {
    match path.rfind('/') {
        Some(0) | None => "/",
        Some(index) => &path[..index],
    }
}

fn find_inode<'a>(fs: &'a RamFs, path: &str) -> Option<&'a MemoryInode> {
    fs.files.iter().find(|inode| inode.name == path)
}

fn has_directory(fs: &RamFs, path: &str) -> bool {
    path == "/"
        || find_inode(fs, path)
            .map(|inode| inode.inode_type == InodeType::Directory)
            .unwrap_or(false)
}

pub fn init() {
    let mut fs = RamFs {
        files: Vec::new(),
        current_dir: String::from("/"),
    };

    for directory in ["/etc", "/dev"] {
        fs.files.push(MemoryInode {
            name: String::from(directory),
            inode_type: InodeType::Directory,
            data: Vec::new(),
        });
    }

    for (name, data) in [
        (
            "/motd",
            b"Welcome to Nyxara Unix-like Operating System!\n".as_slice(),
        ),
        (
            "/readme.txt",
            b"Nyxara kernel v2 with POSIX syscalls and VFS.\n".as_slice(),
        ),
    ] {
        fs.files.push(MemoryInode {
            name: String::from(name),
            inode_type: InodeType::File,
            data: data.to_vec(),
        });
    }

    unsafe {
        *RAMFS.0.get() = Some(fs);
    }
}

pub fn current_dir() -> String {
    unsafe {
        (*RAMFS.0.get())
            .as_ref()
            .map(|fs| fs.current_dir.clone())
            .unwrap_or_else(|| String::from("/"))
    }
}

pub fn change_dir(path: &str) -> Result<(), i32> {
    unsafe {
        let fs = (&mut *RAMFS.0.get()).as_mut().ok_or(-5)?;
        let target = normalize_path(&fs.current_dir, path).ok_or(-2)?;
        if !has_directory(fs, &target) {
            return Err(-2);
        }
        fs.current_dir = target;
        Ok(())
    }
}

pub fn make_dir(path: &str) -> Result<(), i32> {
    unsafe {
        let fs = (&mut *RAMFS.0.get()).as_mut().ok_or(-5)?;
        let target = normalize_path(&fs.current_dir, path).ok_or(-22)?;
        if target == "/" || find_inode(fs, &target).is_some() {
            return Err(-17);
        }
        if !has_directory(fs, parent_path(&target)) {
            return Err(-2);
        }
        fs.files.push(MemoryInode {
            name: target,
            inode_type: InodeType::Directory,
            data: Vec::new(),
        });
        Ok(())
    }
}

pub fn list_dir(path: &str) -> Result<Vec<(String, usize, InodeType)>, i32> {
    unsafe {
        let fs = (&*RAMFS.0.get()).as_ref().ok_or(-5)?;
        let target = normalize_path(&fs.current_dir, path).ok_or(-2)?;
        if !has_directory(fs, &target) {
            return Err(-2);
        }

        let prefix = if target == "/" {
            String::from("/")
        } else {
            let mut value = target.clone();
            value.push('/');
            value
        };
        let mut entries = Vec::new();
        for inode in &fs.files {
            let Some(remainder) = inode.name.strip_prefix(&prefix) else {
                continue;
            };
            if remainder.is_empty() || remainder.contains('/') {
                continue;
            }
            entries.push((String::from(remainder), inode.get_size(), inode.inode_type));
        }
        Ok(entries)
    }
}

pub fn list_files() -> Vec<(String, usize)> {
    list_dir("")
        .unwrap_or_default()
        .into_iter()
        .map(|(name, size, _)| (name, size))
        .collect()
}

pub fn read_file(name: &str) -> Option<Vec<u8>> {
    unsafe {
        let fs = (*RAMFS.0.get()).as_ref()?;
        let path = normalize_path(&fs.current_dir, name)?;
        let inode = find_inode(fs, &path)?;
        if inode.inode_type != InodeType::File {
            return None;
        }
        Some(inode.data.clone())
    }
}

pub fn write_file(name: &str, data: &[u8]) -> Result<(), i32> {
    unsafe {
        let fs = (&mut *RAMFS.0.get()).as_mut().ok_or(-5)?;
        let path = normalize_path(&fs.current_dir, name).ok_or(-22)?;
        if path == "/" || !has_directory(fs, parent_path(&path)) {
            return Err(-2);
        }

        if let Some(inode) = fs.files.iter_mut().find(|inode| inode.name == path) {
            if inode.inode_type != InodeType::File {
                return Err(-21);
            }
            inode.data.clear();
            inode.data.extend_from_slice(data);
            return Ok(());
        }

        fs.files.push(MemoryInode {
            name: path,
            inode_type: InodeType::File,
            data: data.to_vec(),
        });
        Ok(())
    }
}
