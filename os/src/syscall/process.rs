//! Process management syscalls
use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{mm::{translated_byte_buffer, MapPermission, PTEFlags, PageTable, VirtAddr}, sync::UPSafeCell, task::{change_program_brk, current_user_token, exit_current_and_run_next, get_current_task_id, suspend_current_and_run_next, TASK_MANAGER}, timer::get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

unsafe fn copy_to_user<T>(data: &T, addr: usize) {
    let dst_frames: alloc::vec::Vec<&mut [u8]> = translated_byte_buffer(
        current_user_token(),
        addr as *mut u8,
        core::mem::size_of::<T>(),
    );
    let mut start = data as *const T as *const u8;
    for frame in dst_frames {
        let slice = core::slice::from_raw_parts(start, frame.len());
        frame.copy_from_slice(slice);
        start = start.add(frame.len());
    }
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let sec = us / 1_000_000;
    let usec = us % 1_000_000;
    let time_val = TimeVal {
        sec,
        usec,
    };
    unsafe { copy_to_user(&time_val, ts as usize) };
    0
}

fn get_phy_addr(addr: usize, write: bool) -> Option<*mut u8> {
    let page_table = PageTable::from_token(current_user_token());
    let va = VirtAddr::from(addr);
    let Some(pte) = page_table.translate(va.floor()) else {
        error!("kernel: sys_trace: translate failed");
        return None;
    };
    debug!("readable: {}, writable: {}, valid: {}, user: {}", pte.readable(), pte.writable(), pte.is_valid(), pte.is_user());
    if !pte.is_valid() || !pte.is_user() {
        return None;
    }
    if write && !pte.writable() || !write && !pte.readable(){
        return None;
    }
    let ppn = pte.ppn();
    let offset = va.page_offset();
    let bytes = ppn.get_bytes_array();
    let phy_addr = bytes.as_mut_ptr() as usize + offset;
    Some(phy_addr as *mut u8)
}

pub fn sys_trace(_trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        2 => {
            TRACE_DATA.exclusive_access().get(id, get_current_task_id())
        },
        0 => unsafe {
            match get_phy_addr(id, false) {
                Some(p) => core::ptr::read_volatile(p) as isize,
                None => -1
            }
        },
        1 => unsafe {
            match get_phy_addr(id, true) {
                Some(p) => {
                    core::ptr::write_volatile(p, data as u8);
                    0
                }
                None => -1
            }
        },
        _ => -1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if port & !0x7 != 0 || port & 0x7 == 0 {
        error!("kernel: sys_mmap: port value({port:#b}) is invalid");
        return -1;
    }
    match TASK_MANAGER.mmap(
        VirtAddr::from(_start),
        VirtAddr::from(_start + _len),
        MapPermission::from_bits_truncate(
            if port & 0b1 != 0 { PTEFlags::R.bits() } else { 0 } |
            if port & 0b10 != 0 { PTEFlags::W.bits() } else { 0 } |
            if port & 0b100 != 0 { PTEFlags::X.bits() } else { 0 } |
            PTEFlags::U.bits()
        )
    ) {
        Ok(_) => 0,
        Err(e) => {
            error!("kernel: sys_mmap: {e}");
            -1
        }
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    match TASK_MANAGER.munmap(
        VirtAddr::from(_start),
        VirtAddr::from(_start + _len)
    ) {
        Ok(_) => 0,
        Err(e) => {
            error!("kernel: sys_munmap: {e}");
            -1
        }
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

pub struct TraceData {
    records: Vec<(usize, usize, isize)>
}

impl TraceData {
    pub fn new() -> Self {
        TraceData {
            records: Vec::new(),
        }
    }

    pub fn inc(&mut self, syscall_id: usize, task_id: usize) {
        for (s, t, count) in self.records.iter_mut() {
            if *s == syscall_id && *t == task_id {
                *count += 1;
                return;
            }
        }
        self.records.push((syscall_id, task_id, 1));
    }

    pub fn get(&self, syscall_id: usize, task_id: usize) -> isize {
        for (s, t, count) in self.records.iter() {
            if *s == syscall_id && *t == task_id {
                return *count;
            }
        }
        0
    }
}

lazy_static!(
    pub static ref TRACE_DATA: UPSafeCell<TraceData> = unsafe {
        UPSafeCell::new(TraceData::new())
    };
);

pub fn inc_trace(syscall_id: usize) {
    TRACE_DATA.exclusive_access().inc(syscall_id, get_current_task_id());
}