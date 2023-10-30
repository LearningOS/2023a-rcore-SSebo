//! Process management syscalls

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE},
    mm::translated_phisical_address,
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_start_time,
        get_current_status, get_syscall_times, mmap_current_task, munmap_current_task,
        suspend_current_and_run_next, TaskStatus,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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

/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let ts = translated_phisical_address(current_user_token(), ts as *const u8) as *mut TimeVal;
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }

    0
}

/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let ti = translated_phisical_address(current_user_token(), ti as *const u8) as *mut TaskInfo;
    unsafe {
        *ti = TaskInfo {
            status: get_current_status(),
            syscall_times: get_syscall_times(),
            time: (get_time_us() - get_current_start_time()) / 1000,
        };
    }

    0
}

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    if prot & !0x7 != 0 || prot == 0 {
        return -1;
    }

    mmap_current_task(start, len, prot)
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    trace!("kernel: sys_munmap");
    munmap_current_task(start, len)
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
