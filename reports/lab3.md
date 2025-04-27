# 功能总结

1. `sys_spawn`。我的实现方式是先 fork，然后子进程调用 exec，最后返回子进程的 pid。
2. `sys_set_priority`，直接设置优先级即可。需要在 TaskControlBlockInner 结构体中添加两个成员变量，分别是 priority 和 stride。创建新进程的时候，stride 为 0，priority 为 16，fork 的时候需要复制父进程的 priority 和 stride。
3. 调度。修改 `fetch` 函数，从 ready_queue 中找到 stride 最小的进程，将 stride 加上 BIG_STRIDE / priority。然后返回该进程。这样就实现了 stride 调度。

# stride 算法深入

## 实际情况是轮到 p1 执行吗？为什么？

当 p2 执行一个时间片后，stride 值会变成： p2.stride = 250 + (255/10) = 250 + 25 = 275。但由于使用8位无符号整数，最大值为255，所以发生溢出，实际上 p2.stride = 275 - 256 = 19。

比较 p1.stride = 255 和 p2.stride = 19：

- 在普通比较下，19 < 255，会选择 p2 执行
- 但实际上应该选择 p1 执行（因为不考虑溢出时，p1 的 stride 较小）

实际情况是不会轮到 p1 执行，这就是 stride 算法在整数溢出情况下的问题。

## 为什么？尝试简单说明

每次调度时，选择 stride 最小的进程执行。该进程执行后，其 stride 增加 BigStride/priority。由于优先级最低为2，所以每次最多增加 BigStride/2。假设当前最大 stride 为 STRIDE_MAX，最小为 STRIDE_MIN。执行最小 stride 的进程后，它的新值最多为 STRIDE_MIN + BigStride/2。如果这个新值大于 STRIDE_MAX，那么它变成新的最大值。此时新的 STRIDE_MAX - 新的 STRIDE_MIN ≤ BigStride/2。


## 3

```rust
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        const HALF_BIG_STRIDE: u64 = u64::MAX / 2;
        if self.0 == other.0 {
            None
        } else if self.0.wrapping_sub(other.0) > HALF_BIG_STRIDE {
            Some(Ordering::Less)
        } else if other.0.wrapping_sub(self.0) > HALF_BIG_STRIDE {
            Some(Ordering::Greater)
        } else {
            Some(self.0.cmp(&other.0))
        }
    }
}
```