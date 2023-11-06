//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::MAX_SYSCALL_NUM,
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str, translated_ptr, VirtAddr, VirtPageNum},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, mmap, munmap, TaskControlBlock,
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
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    
    let time = get_time_us();
    // println!("--------------------------------------");
    let ts = translated_ptr(current_user_token(), _ts);
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
    trace!(
        "kernel:pid[{}] sys_task_info NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    
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

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    
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
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let path = translated_str(token, _path);

    println!("path: {}", path);

    if let Some(data) = get_app_data_by_name(path.as_str()) {

        // Get the current task 
        let parent_task = current_task().unwrap();
        let mut parent_inner = parent_task.inner_exclusive_access();

        // create a new task control block 
        let child_task = Arc::new(TaskControlBlock::new(data));
        let mut child_inner = child_task.inner_exclusive_access();

        let pid = child_task.getpid();
        // Set proper pointers for the relationship of parents and child process,
        // Inspired by the fork method
        child_inner.parent = Some(Arc::downgrade(&parent_task));
        parent_inner.children.push(child_task.clone());
        drop(child_inner);
        drop(parent_inner);

        add_task(child_task);
        
        // return the child's pid
        pid as isize

    } else {
        println!("Invalid path!");
        -1
    }

}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    
    if _prio < 2 {
        println!("The priority must be greater than 2!");
        return -1;
    }
    
    let current_task = current_task().unwrap();
    let mut inner = current_task.inner_exclusive_access();
    inner.priority = _prio as usize;
    drop(inner);

    _prio
}
