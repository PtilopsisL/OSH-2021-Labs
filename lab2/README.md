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
- 目前**不支持**`cd ~root`、`echo ~root`操作，该输入会出现问题。

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