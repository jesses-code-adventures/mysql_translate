# MySQL Translate

A Rust CLI tool for people that like to modify their mysql databases by hand and
also enjoy using multiple ORMs.

## Why?

When I make a change to my database schema, I want to proliferate that change to
my local projects that consume that database quickly and for multiple formats.

## What's supported?

JSON and Prisma are currently supported, but translators can be built by adding
to src/translators/ (contributions are welcome).

## Navigating the Project

### src/functionality

Here lies the core functionality of MySQL translate, namely the structs for Session,
Database, DiskMapping and AcceptedFormat.

### src/remotes

Could theoretically contain multiple remote data sources, but currently only mysql is
supported in sql.rs.

### src/translators

Specific implementations for each translator. Parsing logic for new translators should
be added as their own file here and added as a variant to the AcceptedFormat variant
in src/functionality. The behaviour.rs file provides the trait implementation
for a new translator.

### src/ui

Specific implementations for different interactivity options. Currently only a janky TUI I made
is available, but I'd like to use clap instead. The behaviour.rs file provides
the trait implementation for a new interactivity option.
