//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
        current_user_token, get_task_info, mmap, munmap
    },
    timer::get_time_us,
    mm::{translate_ptr,  VirtAddr, VirtPageNum}
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
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    let time = get_time_us();
    let ts = translate_ptr(current_user_token(), _ts);
    unsafe {
        *ts = TimeVal {
            sec: time / 1_000_000,
            usec: time % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    
    let ti = translate_ptr(current_user_token(), _ti);
    get_task_info(ti);
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let start_vaddr: VirtAddr = _start.into();

    // 1. _start address is not aligned to a page.

    if !start_vaddr.aligned() {
        println!("start address is not aligned to a page!");
        return -1;
    }

    // 2. _port is not vaild.
    if _port & !0x7 != 0 || _port & 0x7 == 0 {
        println!("Invaild port!");
        return -1;
    }

    // 3. _len is not vaild.
    if _len <= 0 {
        println!("Invaild length!");
        return -1;
    }
    let end_vaddr: VirtAddr = (_start + _len).into();
    let start_vpn: VirtPageNum = start_vaddr.into();
    let end_vpn: VirtPageNum = (end_vaddr).ceil();

    let res = mmap(start_vpn, end_vpn, _port);
    
    // 4. Mapped virtual page exists.
    // 5. Memory used up.

    if res < 0 {
        match res {
            -2 => println!("Some virtual pages has been mapped to a physical page !"),
            -3 => println!("Memory has been used up!"),
            _ => {},
    
       }
       return -1;
    }

     0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");

    let start_vaddr: VirtAddr = _start.into();
    // 1. _start address is not aligned to a page.
    if !start_vaddr.aligned() {
        println!("start address is not aligned to a page!");
        return -1;
    }

    // 2. _len is not vaild.
    if _len <= 0 {
        println!("Invaild length!");
        return -1;
    }
    let end_vaddr: VirtAddr = (_start + _len).into();
    let start_vpn: VirtPageNum = start_vaddr.into();
    let end_vpn: VirtPageNum = (end_vaddr).ceil();

    let res = munmap(start_vpn, end_vpn);

    // 4. Unmapped virtual page exists.
    // 5. Invalid mapped physical page exists.

    if res < 0 {
        match res {
            -2 => println!("Invalid mapped physical page exists."),
            -3 => println!("Unmapped virtual page exists."),
            _ => {},

        }

        return -1;
    }
    
    0

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
