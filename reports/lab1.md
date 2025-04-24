# ch3 实验

为了实现系统调用的记录，我声明了一个结构体和对应的函数，封装了对系统调用 id 和 task id 的次数增加和读操作。另外，为了方便地获取当前运行 task 的 id，也创建了一个函数。

然后在 syscall 函数中，调用了封装好的函数，增加系统调用 id 和 task id 的次数。然后在 sys_trace 中调用函数获取次数。这里的实现我参考了 Task Manager 的代码实现。

修改和读取指针对应的实现只需要使用 `read_volatile` 和 `write_volatile` 函数即可。

# 简答题

## 1

*RustSBI version 0.3.0-alpha.2*

- `ch2b_bad_address.rs`: 内核报错页错误（`PageFault`）；
- `ch2b_bad_instructions.rs`: 内核报错非法指令（`IllegalInstruction`）；
- `ch2b_bad_register.rs`: 内核报错非法指令（`IllegalInstruction`）。

## 2

1. sp 指向了 TrapContext 地址。`__restore` 的两种作用：

    - 当 trap_handler 处理完中断或异常后，调用 __restore 返回到用户态程序继续执行；
    - 第一次进入用户态（内核栈上压入构造好的 Trap 上下文， 然后 __restore）。
2. - sstatus，确保用户态程序在正确的特权级别运行；
   - sepc，决定了 sret 指令将跳转到的用户态程序地址；
   - sscratch，在下面↩恢复用户栈指针。
3. 此处跳过的寄存器在 __alltraps 的注释中有提到：
   - x2 是栈指针，在后面特殊处理
   - x4 是线程指针，当前不使用
4. 这条指令实现了栈指针的原子交换，是特权级切换过程中的关键操作。它让处理器从使用内核栈切换到使用用户栈，完成了从内核态到用户态转换的最后一步。接下来的 sret 指令将正式返回用户态执行环境。
5. `sret`，它会恢复 PC，程序跳转到用户程序，并且根据 sstatus 的值决定返回到哪个特权级别。
6. sp 指向了内核栈空间，sscratch 保存了中断前用户程序正在使用的栈指针
7. 是由处理器自动完成的，在 __alltraps 运行的时候，实际上已经在内核特权级下了