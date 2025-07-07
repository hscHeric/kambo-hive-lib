use log::{debug, error, info, warn};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use uuid::Uuid;

use crate::common::Task;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Completed,
    Failed,
}

pub struct TaskManager {
    pending_tasks: VecDeque<Task>,
    assigned_tasks: HashMap<Uuid, (Task, Uuid)>, // TaskId -> (Task, WorkerId)
    all_tasks_status: HashMap<Uuid, TaskStatus>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            pending_tasks: VecDeque::new(),
            assigned_tasks: HashMap::new(),
            all_tasks_status: HashMap::new(),
        }
    }

    pub fn add_new_graph_tasks(&mut self, graph_id: &str, num_runs: u32, ag_config: &str) {
        info!("Adicionando {num_runs} tasks para o graph {graph_id}");
        for i in 0..num_runs {
            let task = Task::new(graph_id.to_string(), i, ag_config.to_string());
            self.pending_tasks.push_back(task.clone());
            self.all_tasks_status.insert(task.id, TaskStatus::Pending);
        }
        info!("Tasks pendentes: {}", self.pending_tasks.len());
    }

    pub fn get_next_task(&mut self, worker_id: Uuid) -> Option<Task> {
        if let Some(task) = self.pending_tasks.pop_front() {
            info!("Task {} atribuida ao woerker {}", task.id, worker_id);
            self.assigned_tasks
                .insert(task.id, (task.clone(), worker_id));
            self.all_tasks_status.insert(task.id, TaskStatus::Assigned);
            Some(task)
        } else {
            debug!("Não existem tasks pendentes.");
            None
        }
    }

    pub fn mark_task_completed(&mut self, task_id: Uuid) -> Result<(), Box<dyn Error>> {
        if let Some((_, worker_id)) = self.assigned_tasks.remove(&task_id) {
            info!("Task {task_id} finalizada pelo worker {worker_id}");
            self.all_tasks_status.insert(task_id, TaskStatus::Completed);
            Ok(())
        } else {
            warn!("Tentando marcar uma task não atribuida: {task_id}");
            Err(format!("Task {task_id} não foi achada entre as tasks atribuidas").into())
        }
    }

    pub fn mark_task_failed(&mut self, task_id: Uuid) {
        if let Some((task, _)) = self.assigned_tasks.remove(&task_id) {
            error!("Task {task_id} failed, re-adding to pending tasks.");
            self.pending_tasks.push_front(task.clone());
            self.all_tasks_status.insert(task_id, TaskStatus::Failed);
        } else {
            warn!("Tentando marcar uma task não atribuida: {task_id}");
        }
    }

    pub fn get_total_tasks(&self) -> usize {
        self.all_tasks_status.len()
    }

    pub fn get_completed_tasks_count(&self) -> usize {
        self.all_tasks_status
            .values()
            .filter(|&&s| s == TaskStatus::Completed)
            .count()
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
