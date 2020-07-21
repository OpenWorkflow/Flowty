extern crate cron;

use std::str::FromStr;

use cron::Schedule;
use chrono::prelude::*;
use tokio::task::JoinHandle;

use flowty_types::openworkflow;
use super::workflow_instance::{WorkflowInstance, RunState};

pub struct Workflow {
	pub workflow: openworkflow::Workflow,
	schedule: Schedule,
	last_tick: Option<DateTime<Utc>>,
	workflow_instances: Vec<WorkflowInstance>,
}

impl Workflow {
	pub fn new(workflow: openworkflow::Workflow) -> Workflow {
		let schedule = Schedule::from_str(workflow.schedule.as_str().clone()).unwrap();
		Workflow {workflow, schedule, last_tick: None, workflow_instances: Vec::new()}
	}

	pub fn update_workflow(&mut self, openworkflow: openworkflow::Workflow, reset_tick: bool) {
		if openworkflow == self.workflow {
			return;
		}

		if self.workflow.schedule != openworkflow.schedule {
			warn!("Changing schedule. This is can cause undefined behaviour and is not recommend!");
			// ToDo: Actually make it work nicely
			self.schedule = Schedule::from_str(openworkflow.schedule.as_str().clone()).unwrap();
		}
		self.workflow = openworkflow;
		if reset_tick {
			self.last_tick = None;
		}
	}

	pub async fn tick(&mut self, sql_client: &tokio_postgres::Client, now: DateTime<Utc>) {
		if self.last_tick.is_none() {
			trace!("First tick");
			self.last_tick = Some(now);
			return;
		}

		self.run_instances(sql_client).await;
		self.queue_instances(sql_client, now).await;

		self.last_tick = Some(now);
	}

	async fn run_instances(&mut self, sql_client: &tokio_postgres::Client) {
		let queued_instances = self.workflow_instances
			.iter_mut()
			.filter(|i| matches!(i.get_run_state(), RunState::Queued));

		for queued_instance in queued_instances {
			queued_instance.run(sql_client).await;
		}
	}

	async fn queue_instances(&mut self, sql_client: &tokio_postgres::Client, now: DateTime<Utc>) {
		let active_instances: Vec<&WorkflowInstance> = self.workflow_instances
			.iter()
			.filter(|i| matches!(i.get_run_state(), RunState::Queued | RunState::Running))
			.collect();

		if active_instances.len() as u32 >= self.workflow.max_active_runs {
			trace!("Amount of active runs exceeds max active runs for '{}': {} >= {}",
				self.workflow.workflow_id,
				active_instances.len(),
				self.workflow.max_active_runs);
			return;
		}
		let remaining_slots = self.workflow.max_active_runs as usize - active_instances.len();
		trace!("{} remaining slots for workflow '{}'", remaining_slots, self.workflow.workflow_id);

		for instance in self.schedule.after(&self.last_tick.unwrap()).take(remaining_slots) {
			if instance > now {
				break;
			}
			info!("Creating workflow instance for '{}' at time {}", self.workflow.workflow_id, instance);
			if let Ok(mut wi) = WorkflowInstance::new(
				sql_client, &self.workflow.workflow_id, &self.workflow.tasks, instance
				).await {
				wi.queue(sql_client).await;
				self.workflow_instances.push(wi);
			}
		}
	}
}
