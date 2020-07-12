CREATE FUNCTION last_modified_at() RETURNS trigger AS $$
BEGIN
	NEW.modified_at := NOW();

	RETURN NEW;
END;
$$ LANGUAGE plpgsql;