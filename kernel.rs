use std::collections::VecDeque;
use std::fmt;

// --- 1. Core Kernel Definitions ---


#[derive(Debug, Clone, Copy, PartialEq)]
enum ProcessState {
    Ready,      // Waiting to be scheduled
    Running,    // Currently executing on the "CPU"
    Blocked,    // Waiting for an external event (e.g., I/O - simplified here)
    Exited,     // Finished execution
}


#[derive(Clone)]
struct Process {
    id: u32,
    name: String,
    state: ProcessState,
    program_counter: u32, 
    priority: u8,        
}

impl Process {

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


impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Task {}] ({}): State={:?}, PC={}",
            self.id, self.name, self.state, self.program_counter
        )
    }
}


#[derive(Debug)]
enum KernelCall {
    Yield,      
    Print(String), 
    Exit,       
    Block,      
}

// --- 2. The Kernel Structure and Logic ---


struct Kernel {
    next_pid: u32,
    ready_queue: VecDeque<Process>,
    running_task: Option<Process>,  
    ticks: u32,                     
}

impl Kernel {

    fn new() -> Self {
        Kernel {
            next_pid: 1,
            ready_queue: VecDeque::new(),
            running_task: None,
            ticks: 0,
        }
    }

   
    fn spawn_task(&mut self, name: &str, priority: u8) {
        let task = Process::new(self.next_pid, name, priority);
        println!("[KERNEL] Spawning: {}", task);
        self.ready_queue.push_back(task);
        self.next_pid += 1;
    }

   
    fn schedule(&mut self) {
        
        if let Some(mut current_task) = self.running_task.take() {
       
            if current_task.state == ProcessState::Running {
                current_task.state = ProcessState::Ready;
                println!(
                    "[SCHEDULER] Time slice ended for {}. Re-queuing.",
                    current_task.id
                );
                self.ready_queue.push_back(current_task);
            }
        }

        
        if let Some(mut next_task) = self.ready_queue.pop_front() {
            next_task.state = ProcessState::Running;
            println!("[SCHEDULER] Dispatching: {}", next_task);
            self.running_task = Some(next_task);
        } else {
           
            println!("[SCHEDULER] Ready queue empty. Idling.");
        }
    }

   
    fn tick(&mut self) -> bool {
        self.ticks += 1;
        println!("\n--- TICK {} ---", self.ticks);

      
        if self.ticks % 3 == 0 {
            self.schedule();
        }

      
        if let Some(ref mut task) = self.running_task {
      
            task.program_counter += 1;
            println!("[CPU] Running: {}. PC: {}", task.id, task.program_counter);

           
            let kernel_call = self.simulate_task_logic(task.id, task.program_counter);

            if let Some(call) = kernel_call {
                self.handle_kernel_call(call);
            }

        } else {
           
            if self.ready_queue.is_empty() {
                println!("[KERNEL] All tasks completed. Shutting down.");
                return false; 
            }
         
            self.schedule();
        }

        true 
    }

   
    fn simulate_task_logic(&self, task_id: u32, pc: u32) -> Option<KernelCall> {
        match task_id {
           
            1 => match pc {
                5 => Some(KernelCall::Print(format!("Task {} is halfway!", task_id))),
                10 => Some(KernelCall::Exit),
                _ => None,
            },
           
            2 => match pc {
                3 => Some(KernelCall::Yield),
                8 => Some(KernelCall::Print(format!("Task {} is doing work.", task_id))),
                12 => Some(KernelCall::Exit),
                _ => None,
            },
           
            3 => match pc {
                4 => Some(KernelCall::Block),
                _ => None,
            },
            _ => None,
        }
    }

   
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
                  
                }
                self.schedule();
            }
        }
    }
}

fn main() {
    println!("--- Kernel Simulation Start ---");
    let mut kernel = Kernel::new();

   
    kernel.spawn_task("Init_Task", 10);
    kernel.spawn_task("WebApp_Worker", 5);
    kernel.spawn_task("File_IO_Task", 8);

 
    kernel.schedule();


    while kernel.tick() && kernel.ticks < 20 {
       
    }

    println!("\n--- Simulation End ---");
    println!("Total Ticks: {}", kernel.ticks);

   
    println!("\n--- Final Task State ---");
    if let Some(task) = kernel.running_task.as_ref() {
        println!("{}", task);
    }
    for task in kernel.ready_queue.iter() {
        println!("{}", task);
    }
   
    println!("\n(Note: Exited and Blocked tasks are no longer tracked in the queues.)");
}