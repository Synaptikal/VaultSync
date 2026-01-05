use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Type alias for a task generator function.
/// It returns a Future, which allows us to re-create the async task when restarting.
type TaskGenerator = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

struct SupervisedTask {
    name: String,
    generator: TaskGenerator,
    handle: Option<JoinHandle<()>>,
    restart_count: u32,
}

/// ServiceSupervisor manages long-running background tasks.
/// It monitors them and automatically restarts them if they crash (panic) or exit unexpectedly.
#[derive(Clone)]
pub struct ServiceSupervisor {
    tasks: Arc<Mutex<Vec<SupervisedTask>>>,
}

impl ServiceSupervisor {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a task to be supervised.
    ///
    /// * `name`: Human-readable name for logging
    /// * `task_generator`: A closure that returns the async block to run.
    ///                   This closure is called initially and on every restart.
    pub async fn spawn<F, Fut>(&self, name: &str, task_generator: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let generator_box: TaskGenerator = Box::new(move || Box::pin(task_generator()));

        // Spawn the initial instance
        let handle = tokio::spawn((generator_box)());
        tracing::info!("Supervisor: Started task '{}'", name);

        let task = SupervisedTask {
            name: name.to_string(),
            generator: generator_box,
            handle: Some(handle),
            restart_count: 0,
        };

        let mut tasks = self.tasks.lock().await;
        tasks.push(task);
    }

    /// Start the monitoring loop. This should be spawned as its own task.
    /// It checks all supervised tasks periodically.
    pub async fn monitor(self) {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let mut tasks = self.tasks.lock().await;
            for task in tasks.iter_mut() {
                let needs_restart = if let Some(handle) = &task.handle {
                    handle.is_finished()
                } else {
                    true
                };

                if needs_restart {
                    task.restart_count += 1;
                    tracing::warn!(
                        "Supervisor: Task '{}' exited unexpectedly. Restarting... (Attempt {})",
                        task.name,
                        task.restart_count
                    );

                    // Re-spawn
                    let new_future = (task.generator)();
                    task.handle = Some(tokio::spawn(new_future));
                }
            }
        }
    }
}
