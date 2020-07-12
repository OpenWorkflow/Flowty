DROP TABLE IF EXISTS workflow_instance;

CREATE TABLE IF NOT EXISTS workflow_instance (
	wiid SERIAL PRIMARY KEY,
	workflow_id TEXT,
	run_state runstate DEFAULT 'nothing',
	run_date TIMESTAMP,

	created_at TIMESTAMP DEFAULT NOW(),
	modified_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_workflow_instance_workflow_id ON workflow_instance(workflow_id);

CREATE TRIGGER last_modified_at
BEFORE UPDATE ON workflow_instance
FOR EACH ROW EXECUTE PROCEDURE last_modified_at();
