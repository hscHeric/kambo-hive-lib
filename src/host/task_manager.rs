use std::{
    collections::{HashMap, VecDeque},
    error::Error,
};

use log::{debug, error, info, warn};
use rand::seq::IndexedRandom;
use uuid::Uuid;

use crate::common::Task;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Completed,
    Failed,
}

// NOVO: Enum para definir a estratégia de distribuição
#[derive(Debug, Clone, Copy)]
pub enum DistributionStrategy {
    Fifo, // First-In, First-Out
    Lifo, // Last-In, First-Out
    Random,
}

pub struct TaskManager {
    pending_tasks: VecDeque<Task>,
    assigned_tasks: HashMap<Uuid, (Task, Uuid)>, // TaskId -> (Task, WorkerId)
    all_tasks_status: HashMap<Uuid, TaskStatus>,
    distribution_strategy: DistributionStrategy, // NOVO: Campo para a estratégia
}

impl TaskManager {
    // ATUALIZADO: `new` agora aceita uma estratégia de distribuição
    pub fn new(distribution_strategy: DistributionStrategy) -> Self {
        Self {
            pending_tasks: VecDeque::new(),
            assigned_tasks: HashMap::new(),
            all_tasks_status: HashMap::new(),
            distribution_strategy,
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

    // ATUALIZADO: `get_next_task` agora usa a estratégia de distribuição
    pub fn get_next_task(&mut self, worker_id: Uuid) -> Option<Task> {
        let task = match self.distribution_strategy {
            DistributionStrategy::Fifo => self.pending_tasks.pop_front(),
            DistributionStrategy::Lifo => self.pending_tasks.pop_back(),
            DistributionStrategy::Random => {
                if self.pending_tasks.is_empty() {
                    None
                } else {
                    let mut rng = rand::rng();
                    let index = *(0..self.pending_tasks.len())
                        .collect::<Vec<usize>>()
                        .choose(&mut rng)
                        .unwrap();
                    self.pending_tasks.remove(index)
                }
            }
        };

        if let Some(task) = task {
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
            error!("Task {task_id} falhou");
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

    pub fn get_tasks_status(&self) -> &HashMap<Uuid, TaskStatus> {
        &self.all_tasks_status
    }
}

