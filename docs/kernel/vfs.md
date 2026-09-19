# Virtual File System (VFS) & In-Memory RamFS

The Virtual File System (`rust/src/vfs.rs`) provides a Unix-like file and directory abstraction in Nyxara OS. It abstracts file storage operations behind standard traits, allowing in-memory filesystems (RamFS), device nodes, and future block-device filesystems (ext2/FAT) to share a uniform interface.

## Inode Abstraction

```rust
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
```

## In-Memory File System (`RamFs`)

Files in Nyxara are currently held in a RAM-backed filesystem using dynamic byte vectors.

### Data Structure (`MemoryInode`)
```rust
pub struct MemoryInode {
    pub name: String,
    pub inode_type: InodeType,
    pub data: Vec<u8>,
}
```

- Reading checks offset bounds and copies the available slice to the caller buffer.
- Writing automatically expands the backing `Vec<u8>` if the offset plus payload exceeds current length.

### Default System Files
During `vfs::init()`, default files are created:
- **`motd`**: Message of the day ("Welcome to Nyxara Unix-like Operating System!").
- **`readme.txt`**: Kernel release and architecture notes.

## File System APIs

```rust
// List all files and sizes in the root filesystem
pub fn list_files() -> Vec<(String, usize)>

// Read the complete contents of a named file into a byte buffer
pub fn read_file(name: &str) -> Option<Vec<u8>>

// Write or overwrite an existing file, or create a new file if not found
pub fn write_file(name: &str, data: &[u8]) -> Result<(), i32>
```

## Shell and Application Integration

VFS operations are exposed directly to the interactive shell:
- **`ls`**: Displays all files and file sizes.
- **`cat <file>`**: Reads and prints the text content of a file.
- **`touch <file>`**: Creates an empty file in RamFS.
- **`write <file> <text>`**: Appends or overwrites text in a file.
- **`mway <file>`**: Loads the file into the full-screen visual editor and commits changes back to VFS upon pressing `Ctrl+S`.
