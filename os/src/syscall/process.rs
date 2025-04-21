//! Process management syscalls
use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{
    sync::UPSafeCell, task::{exit_current_and_run_next, get_current_task_id, suspend_current_and_run_next}, timer::get_time_us
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
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

pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        2 => {
            TRACE_DATA.exclusive_access().get(_id, get_current_task_id())
        },
        0 => unsafe {
            core::ptr::read_volatile(_id as *const u8) as isize
        },
        1 => unsafe {
            core::ptr::write_volatile(_id as *mut u8, _data as u8);
            0
        },
        _ => -1
    }
}
