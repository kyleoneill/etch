# Table Storage
Tables are currently stored in a single `tables.etch` file. This is fine for a small database or if tables do not
need to be updated, but if tables got larger and the ability to update tables were added, read/writes to this file
would become a problem.

An alternate, and better, way to store tables would be to have `tables.etch` store a list of file names, where
each table got its own metadata file. That way, appending a new table is _just_ adding a new line of the file name
to `tables.etch` and changes or full file re-writes could be made easily to a metadata file for a specific table.
`tables.etch` and all table files can live under a `tables` directory.

# Row Storage
Rows are stored in sub_table files. A row has an ID that takes the form of `{usize}.{uuid}` where
the first segment is a usize indicating which sub-table file a row is stored in, and the second
segment is a UUID.

If a row object in the 'foo' table is accessed by some ID that looks like `4.ABC-123-456` then the
record will be in the /db_files/foo/sub_table_4.etch file. This schema works fine when working with
objects by ID or without many concurrent requests but this does not scale or work if access is made
by means other than ID

# Concurrency

# Frame Serialization
