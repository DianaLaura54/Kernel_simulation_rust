use std::collections::VecDeque;
use std::fmt;

// --- 1. Core Kernel Definitions ---

/// Represents the possible states of a task/process managed by the kernel.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ProcessState {
    Ready,      // Waiting to be scheduled
    Running,    // Currently executing on the "CPU"
    Blocked,    // Waiting for an external event (e.g., I/O - simplified here)
    Exited,     // Finished execution
}

/// Represents a task or process.
#[derive(Clone)]
struct Process {
    id: u32,
    name: String,
    state: ProcessState,
    program_counter: u32, // Represents how far along the task is in its execution
    priority: u8,         // Simple priority field (unused in this Round Robin example, but common)
}

impl Process {
    /// Creates a new, ready process.
    fn new(id: u32, name: &str, priority: u8) -> Self {
        Process {
            id,
            name: name.to_string(),
            state: ProcessState::Ready,
            program_counter: 0,
            priority,
        }
    }
}

// Implement a custom display trait for nice printing
impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Task {}] ({}): State={:?}, PC={}",
            self.id, self.name, self.state, self.program_counter
        )
    }
}

/// System calls that a running process can make to request kernel services.
#[derive(Debug)]
enum KernelCall {
    Yield,      // Relinquish the CPU to another task
    Print(String), // Request the kernel to output a message
    Exit,       // Terminate the process
    Block,      // Go into a waiting state (e.g., waiting for I/O)
}

// --- 2. The Kernel Structure and Logic ---

/// The central structure managing all processes and scheduling.
struct Kernel {
    next_pid: u32,
    ready_queue: VecDeque<Process>, // Stores processes ready to run
    running_task: Option<Process>,  // The task currently holding the CPU
    ticks: u32,                     // Global timer for the simulation
}

impl Kernel {
    /// Initializes a new kernel instance.
    fn new() -> Self {
        Kernel {
            next_pid: 1,
            ready_queue: VecDeque::new(),
            running_task: None,
            ticks: 0,
        }
    }

    /// Adds a new task to the kernel's management.
    fn spawn_task(&mut self, name: &str, priority: u8) {
        let task = Process::new(self.next_pid, name, priority);
        println!("[KERNEL] Spawning: {}", task);
        self.ready_queue.push_back(task);
        self.next_pid += 1;
    }

    /// The core scheduling logic (Simple Round Robin).
    fn schedule(&mut self) {
        // If a task was running, check if it needs to be put back in the queue
        if let Some(mut current_task) = self.running_task.take() {
            // Only re-queue if the task is still running (i.e., it didn't call Exit)
            if current_task.state == ProcessState::Running {
                current_task.state = ProcessState::Ready;
                println!(
                    "[SCHEDULER] Time slice ended for {}. Re-queuing.",
                    current_task.id
                );
                self.ready_queue.push_back(current_task);
            }
        }

        // Pick the next task from the ready queue
        if let Some(mut next_task) = self.ready_queue.pop_front() {
            next_task.state = ProcessState::Running;
            println!("[SCHEDULER] Dispatching: {}", next_task);
            self.running_task = Some(next_task);
        } else {
            // If the ready queue is empty, the kernel is idle
            println!("[SCHEDULER] Ready queue empty. Idling.");
        }
    }

    /// Executes one time slice (one step) of the currently running task.
    fn tick(&mut self) -> bool {
        self.ticks += 1;
        println!("\n--- TICK {} ---", self.ticks);

        // Every 3 ticks, force a schedule (preemption)
        if self.ticks % 3 == 0 {
            self.schedule();
        }

        // Check if there is a task to run
        if let Some(ref mut task) = self.running_task {
            // Simulate task execution progress
            task.program_counter += 1;
            println!("[CPU] Running: {}. PC: {}", task.id, task.program_counter);

            // Simulate task's "program" logic and potential kernel calls
            let kernel_call = self.simulate_task_logic(task.id, task.program_counter);

            if let Some(call) = kernel_call {
                self.handle_kernel_call(call);
            }

        } else {
            // If the ready queue is empty and no task is running, the simulation is done
            if self.ready_queue.is_empty() {
                println!("[KERNEL] All tasks completed. Shutting down.");
                return false; // Stop the simulation
            }
            // If no task is running but the queue isn't empty, schedule immediately
            self.schedule();
        }

        true // Continue simulation
    }

    /// Simulates a task's internal logic and determines if it makes a kernel call.
    fn simulate_task_logic(&self, task_id: u32, pc: u32) -> Option<KernelCall> {
        match task_id {
            // Task 1: Runs for a while then exits
            1 => match pc {
                5 => Some(KernelCall::Print(format!("Task {} is halfway!", task_id))),
                10 => Some(KernelCall::Exit),
                _ => None,
            },
            // Task 2: Runs, yields, and runs more, then exits
            2 => match pc {
                3 => Some(KernelCall::Yield),
                8 => Some(KernelCall::Print(format!("Task {} is doing work.", task_id))),
                12 => Some(KernelCall::Exit),
                _ => None,
            },
            // Task 3: Runs a bit, then blocks (simulating I/O wait)
            3 => match pc {
                4 => Some(KernelCall::Block),
                _ => None,
            },
            _ => None,
        }
    }

    /// Handles a request from a user process to the kernel.
    fn handle_kernel_call(&mut self, call: KernelCall) {
        match call {
            KernelCall::Yield => {
                println!("[KERNEL] Task requested a Yield.");
                self.schedule();
            }
            KernelCall::Exit => {
                if let Some(mut task) = self.running_task.take() {
                    task.state = ProcessState::Exited;
                    println!("[KERNEL] Task {} EXITED.", task.id);
                }
            }
            KernelCall::Print(msg) => {
                if let Some(ref task) = self.running_task {
                    println!("[KERNEL/OUT] Task {} says: {}", task.id, msg);
                }
            }
            KernelCall::Block => {
                if let Some(mut task) = self.running_task.take() {
                    task.state = ProcessState::Blocked;
                    println!("[KERNEL] Task {} BLOCKED. Requires a new schedule.", task.id);
                    // In a real OS, a separate mechanism would unblock it. Here, it's just removed.
                }
                self.schedule();
            }
        }
    }
}

fn main() {
    println!("--- Kernel Simulation Start ---");
    let mut kernel = Kernel::new();

    // Spawn initial tasks
    kernel.spawn_task("Init_Task", 10);
    kernel.spawn_task("WebApp_Worker", 5);
    kernel.spawn_task("File_IO_Task", 8);

    // Initial schedule to get the first task running
    kernel.schedule();

    // Run the main simulation loop for a maximum of 20 ticks
    while kernel.tick() && kernel.ticks < 20 {
        // Delay for visual separation between ticks
        // Note: In a real kernel, this loop runs continuously at high speed.
    }

    println!("\n--- Simulation End ---");
    println!("Total Ticks: {}", kernel.ticks);

    // Final state of tasks (only showing what's left in the queue/running)
    println!("\n--- Final Task State ---");
    if let Some(task) = kernel.running_task.as_ref() {
        println!("{}", task);
    }
    for task in kernel.ready_queue.iter() {
        println!("{}", task);
    }
    // Blocked and Exited tasks are not explicitly tracked in this simple model once removed.
    println!("\n(Note: Exited and Blocked tasks are no longer tracked in the queues.)");
}