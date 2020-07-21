WITH latest AS (
	SELECT MAX(wid) AS wid, workflow_id FROM workflow GROUP BY workflow_id
)
SELECT workflow.workflow_id, workflow.openworkflow_message
FROM workflow JOIN latest ON workflow.wid = latest.wid;