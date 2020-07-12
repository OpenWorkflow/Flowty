CREATE TYPE runstate AS ENUM (
	'nothing',
	'queued',
	'running',
	'success',
	'failed',
);
