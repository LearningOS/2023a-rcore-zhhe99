//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;

// ch3 编写代码 start
/// balaba
#[derive(Copy, Clone)]
pub struct TaskInfoInner {
    /// balabala
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// balabala
    pub start_time: usize,
}

// ch3 编写代码 end

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    // ch3 编写代码 start
    /// balaba
    pub task_info_inner: TaskInfoInner,
    // ch3 编写代码 end
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
