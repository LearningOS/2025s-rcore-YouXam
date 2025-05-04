# 功能总结

1. `linkat`
    - 首先在 `DiskInode` 中添加一个成员变量 `link_count`，用于记录链接数。
    - 在 `linkat` 函数中，先找到旧文件的 block，增加 `link_count`。然后在根目录中创建一个新的目录项，指向旧文件的 inode。这里要注意 inode_id 和 block_id 的区别。
2. `unlinkat`
    - 在 `unlinkat` 函数中，先找到旧文件的 block，减少 `link_count`。如果 `link_count` 为 0，则删除该 inode。删除的时候参考 clear 函数，首先重置 size 为 0，然后释放所有 data 块，最后释放 inode 块。`dealloc_inode` 函数是参考 `dealloc_data` 函数实现的。
    - 然后在根目录中找到旧文件的目录项，如果目录项不是最后一个，就把最后一个目录项复制到旧文件目录项的位置，然后调整目录项的数量。
3. `fstat`
    - 首先给 File 添加了一个方法 fstat，对于 Stdin 和 Stdout 返回默认值。对于 OSInode 则调用 Inode 的 stat 方法，读取类型和 link 数量。
    - 根据文件描述符可以找到对应的 OSInode，调用 fstat 方法，返回一个 Stat 结构体。
    - 然后调用之前实现的 copy_to_user 方法，将 Stat 结构体复制到用户空间。

# 问答作业（ch6）

> 在我们的easy-fs中，root inode起着什么作用？如果root inode中的内容损坏了，会发生什么？

root inode 是访问整个文件系统的起点，所有文件和目录的查找都从此开始。存储了根目录的内容，包含了文件系统中第一级别的所有文件和目录的信息。

如果 root inode 的内容损坏，即使文件数据本身还在磁盘上，但因为无法索引，相当于无法访问。

# 问答作业（ch7）

## a

> 举出使用 pipe 的一个实际应用的例子。
> tips:
> - 想想你平时咋使用 linux terminal 的？
> - 如何使用 cat 和 wc 完成一个文件的行数统计？

使用管道进行文件行数统计

```sh
cat file.txt | wc -l
```

或者使用 grep 查找命令输出

```sh
ps aux | grep python
```

## b

> 如果需要在多个进程间互相通信，则需要为每一对进程建立一个管道，非常繁琐，请设计一个更易用的多进程通信机制。

使用消息总线来传递消息。

- 创建一个中央进程作为消息转发中心，维护一个消息队列和进程注册表
- 其他进程注册到中央进程，发送消息时指定目标进程 ID
- 中央进程接收消息并转发到目标进程
- 进程可以订阅感兴趣的消息类型，中央进程只转发相关消息