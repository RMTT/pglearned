# pgl - pglearned CLI

`pgl` is a command-line interface tool designed to interact with the `pglearned` PostgreSQL extension. It helps manage query datasets and collect execution plans for machine learning tasks.

## Prerequisites

- Rust (latest stable)
- PostgreSQL database with the `pglearned` extension installed.

## Installation

```bash
cd cli
cargo build --release
```

The binary will be available at `target/release/pgl`.

## Configuration

You can specify the database connection URL in two ways:

1.  **Environment Variable**:
    ```bash
    export DATABASE_URL="postgres://user:password@localhost:5432/dbname"
    ```
2.  **Command Line Argument**:
    ```bash
    pgl --db-url "postgres://user:password@localhost:5432/dbname" <COMMAND>
    ```

## Logging

The tool uses `env_logger`. The default log level is `info`. You can override this by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug pgl ...
RUST_LOG=error pgl ...
```

## Usage

### Manage Query Datasets

All dataset commands are grouped under the `qdataset` subcommand.

#### List Datasets
List all available datasets and their current processing position.

```bash
pgl qdataset ls
```

#### Create a Dataset
Create a new empty dataset.

```bash
pgl qdataset create <dataset_name>
```

#### Import Queries
Import queries from a file located **on the database server**.

```bash
pgl qdataset import <dataset_name> <sql_file_path>
```

#### Insert a Query
Insert a single SQL query string into a dataset.

```bash
pgl qdataset insert <dataset_name> "SELECT * FROM my_table"
```

#### Delete a Dataset
Delete a dataset and its associated status.

```bash
pgl qdataset delete <dataset_name>
```

#### Run/Collect Dataset
Run queries in a dataset, collect their execution plans (via `pgl_qdataset_collect`), and save the results to a JSON file.

```bash
pgl qdataset run <dataset_name> --out <output_file.json> [OPTIONS]
```

**Options:**

*   `--out <file>`: (Required) Path to the output JSON file.
*   `--continue`: (Optional) Continue from the last saved position (offset -1) instead of starting from the beginning.
*   `--method <method>`: (Optional) Specify the collection method (default: "default").

**Examples:**

Run from the beginning using the default method:
```bash
pgl qdataset run my_dataset --out plans.json
```

Continue from where it left off:
```bash
pgl qdataset run my_dataset --out plans_part2.json --continue
```

Run using a specific method (e.g., 'exhaustive'):
```bash
pgl qdataset run my_dataset --out plans.json --method exhaustive
```
