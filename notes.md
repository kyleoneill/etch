# Table Storage
Tables are currently stored in a single `tables.etch` file. This is fine for a small database or if tables do not
need to be updated, but if tables got larger and the ability to update tables were added, read/writes to this file
would become a problem.

An alternate, and better, way to store tables would be to have `tables.etch` store a list of file names, where
each table got its own metadata file. That way, appending a new table is _just_ adding a new line of the file name
to `tables.etch` and changes or full file re-writes could be made easily to a metadata file for a specific table.
`tables.etch` and all table files can live under a `tables` directory.

# Concurrency

# Frame Serialization
