# Lab 2
## 编写Shell程序
### 编译运行
在lab2目录下：

编译：
```bash
cargo build
```
运行：
```bash
cargo run
```

### 更健壮
- 支持`cd ~`、`cd`（示例shell代码似乎并不支持`cd ~`，与文档说明不符）。
- 目前**不支持**`cd ~root`、`echo ~root`操作，，若执行`~(some user)`会出现问题。

### 支持管道

### 支持基本的文件重定向

### 处理Ctrl-C的按键

### 处理Ctrl-D的按键

最终表现可能因为`shell`不同而不同，例如用`zsh`运行：
```text
VM2531-lhy% cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/shell`
[ubuntu@VM2531-lhy] lab2 $ %                              
VM2531-lhy% 
```
`bash`运行：
```text
ubuntu@VM2531-lhy:/home/ubuntu/OSH-2021-Labs/lab2$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/shell`
[ubuntu@VM2531-lhy] lab2 $ ubuntu@VM2531-lhy:/home/ubuntu/OSH-2021-Labs/lab2$ 
```
按下`Ctrl-D`后，`zsh`末尾有一个奇怪的`%`符号，并且带换行；而`bash`不带换行。考虑到很难支持不同`shell`的行为，本`shell`在处理`Ctrl-D`的时候不主动换行。
### 更多功能
- 支持`echo $SHELL`类操作（即`echo $(some env)`）。

## strace追踪系统调用
追踪编写的shell程序，结果如下：

```bash
execve("target/debug/shell", ["target/debug/shell"], 0x7ffc698a8840 /* 35 vars */) = 0
brk(NULL)                               = 0x55fe17037000
arch_prctl(0x3001 /* ARCH_??? */, 0x7ffcf48356c0) = -1 EINVAL (无效的参数)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (没有那个文件或目录)
openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
fstat(3, {st_mode=S_IFREG|0644, st_size=84039, ...}) = 0
mmap(NULL, 84039, PROT_READ, MAP_PRIVATE, 3, 0) = 0x7fe08a035000
close(3)                                = 0
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/libdl.so.2", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0 \22\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=18816, ...}) = 0
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x7fe08a033000
mmap(NULL, 20752, PROT_READ, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x7fe08a02d000
mmap(0x7fe08a02e000, 8192, PROT_READ|PROT_EXEC, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x1000) = 0x7fe08a02e000
mmap(0x7fe08a030000, 4096, PROT_READ, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x3000) = 0x7fe08a030000
mmap(0x7fe08a031000, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_FIXED|MAP_DENYWRITE, 3, 0x3000) = 0x7fe08a031000
close(3)                                = 0
openat(AT_FDCWD, "/lib/x86_64-linux-gnu/librt.so.1", O_RDONLY|O_CLOEXEC) = 3
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0 7\0\0\0\0\0\0"..., 832) = 832
fstat(3, {st_mode=S_IFREG|0644, st_size=40040, ...}) = 0
mmap(NULL, 44000, PROT_READ, MAP_PRIVATE|MAP_DENYWRITE, 3, 0) = 0x7fe08a022000
mprotect(0x7fe08a025000, 24576, PROT_NONE) = 0
...
sigaltstack({ss_sp=NULL, ss_flags=SS_DISABLE, ss_size=8192}, NULL) = 0
munmap(0x7fe08a047000, 12288)           = 0
exit_group(0)                           = ?
+++ exited with 0 +++
```

### `mmap`
定义：
```C
#include <sys/mman.h>
void *mmap(void *addr, size_t length, int prot, int flags,
            int fd, off_t offset);
```
作用：

`mmap`是一种内存映射文件的方法，即将一个文件或者其它对象映射到进程的地址空间，实现文件磁盘地址和进程虚拟地址空间中一段虚拟地址的一一对应关系。实现这样的映射关系后，进程就可以采用指针的方式读写操作这一段内存，而系统会自动回写脏页面到对应的文件磁盘上，即完成了对文件的操作而不必再调用`read`，`write`等系统调用函数。

### `mprotect`
定义：
```C
#include <sys/mman.h>
int mprotect(void *addr, size_t len, int prot);
```
作用：
可以用来修改一段指定内存区域的保护属性。

保护属性有：
```text
PROT_NONE
        The memory cannot be accessed at all.

PROT_READ
        The memory can be read.

PROT_WRITE
        The memory can be modified.

PROT_EXEC
        The memory can be executed.
...
```
等等。

### `access`
定义：
```C
#include <unistd.h>
int access(const char *pathname, int mode);
```
作用：
检查当前进程是否有对某个文件的访问权限（`rwx`权限）。

### `fstat`
定义：
```C
#include <sys/types.h>
#include <sys/stat.h>
int fstat(int fd, struct stat *statbuf);
```
作用：
读取一个文件描述符的相关信息。

相关信息的结构体定义为：
```C
struct stat {
    dev_t     st_dev;         /* ID of device containing file */
    ino_t     st_ino;         /* Inode number */
    mode_t    st_mode;        /* File type and mode */
    nlink_t   st_nlink;       /* Number of hard links */
    uid_t     st_uid;         /* User ID of owner */
    gid_t     st_gid;         /* Group ID of owner */
    dev_t     st_rdev;        /* Device ID (if special file) */
    off_t     st_size;        /* Total size, in bytes */
    blksize_t st_blksize;     /* Block size for filesystem I/O */
    blkcnt_t  st_blocks;      /* Number of 512B blocks allocated */

    /* Since Linux 2.6, the kernel supports nanosecond
        precision for the following timestamp fields.
        For the details before Linux 2.6, see NOTES. */

    struct timespec st_atim;  /* Time of last access */
    struct timespec st_mtim;  /* Time of last modification */
    struct timespec st_ctim;  /* Time of last status change */

#define st_atime st_atim.tv_sec      /* Backward compatibility */
#define st_mtime st_mtim.tv_sec
#define st_ctime st_ctim.tv_sec
};
```