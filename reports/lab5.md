# 功能总结

在 `ProcessControlBlockInner` 创建 `pub deadlock_detect: Option<(DeadLockDetect, DeadLockDetect)>`，如果为 None，表示没有启用死锁检测；如果为 Some，表示启用死锁检测。该元组中第一个为 Mutex 的死锁检测，第二个为 Semaphore 的死锁检测。初始值设为 None。

创建一个 `ProcessControlBlock` 的 `enable_deadlock_detect` 方法，当启用死锁检测的时候，就创建对应的 `DeadLockDetect` 实例。

在 `DeadLockDetect` 中实现死锁检测逻辑和银行家算法。包含：

1. `add_resource` 方法：添加一种新的资源，并初始化资源的数量。
2. `test` 方法，实现银行家算法，检测当前状态是否安全。
3. `resize_thread` 方法：调整线程的数量，如果创建了多个线程，使用该方法调整 Allocation 和 Need 矩阵大小。
4. `request` 方法，请求资源并检测死锁，最后更新 Allocation 和 Need 矩阵。
5. `release` 方法，释放资源并更新 Allocation 和 Need 矩阵。允许 release 方法释放比当前分配的更多的资源，在 Semaphore 中有意义。

# 问答作业

1. 当主线程退出时，需要回收的资源包括线程的栈内存、TaskControlBlock（、动态分配的资源（如文件描述符）以及同步原语（如锁和信号量）。TCB 可能被进程的线程列表、调度器队列、同步原语的等待队列或其他线程引用。所有这些引用都需要清理，以避免资源泄漏或死锁问题。

2. 对比以下两种 Mutex 中的实现，二者有什么区别？这些区别可能会导致什么问题？

Mutex1 使用 loop 循环不断尝试获取锁，直到成功为止。Mutex2 没有循环逻辑，只尝试一次获取锁。Mutex1 在解锁时总是将 locked 设置为 false。Mutex2 只有在等待队列为空时才将 locked 设置为 false。


Mutex1 的循环逻辑可能导致忙等，浪费 CPU 资源。Mutex2 在没有循环时，可能导致锁未被正确释放，进而引发死锁或资源无法被其他线程获取的问题。
