# pglearned extension

`pglearned` is a PostgreSQL extension that integrates machine learning capabilities into the query process. It allows for dataset management, data collection, and replacing some components.

## Dataset Management

pglearned supports managing SQL queries as datasets to collect metrics for training or evaluation.

### Functions

*   **Create a dataset**:
    ```sql
    select pgl_qdataset_create('dataset_name');
    ```

*   **Delete a dataset**:
    ```sql
    select pgl_qdataset_delete('dataset_name');
    ```

*   **Add a single query to a dataset**:
    ```sql
    select pgl_qdataset_insert('dataset_name', 'SELECT * FROM my_table');
    ```

*   **Import queries from a file** (semicolon separated):
    ```sql
    select pgl_qdataset_import('dataset_name', '/absolute/path/to/queries.sql');
    ```

## Collecting Data

You can execute queries in a dataset and capture their execution plans and metrics using `pgl_qdataset_collect`.

```sql
-- Signature
pgl_qdataset_collect(
    dataset_name text,
    offset bigint,
    limit bigint,
    method text DEFAULT 'default',
    arm integer DEFAULT -1
) returns table (id integer, plan jsonb)
```

Examples:
```sql
-- Run next 64 queries starting from offset 0 using standard planner
select plan from pgl_qdataset_collect('imdb', 0, 64);

-- Continue from the last recorded position, run 10 queries
select plan from pgl_qdataset_collect('imdb', -1, 10);

-- Use 'brute' method to explore all planner configurations (arms) for the next 10 queries
-- arm = -1 means iterate all possible combinations (2^6 = 64 arms)
select plan from pgl_qdataset_collect('imdb', -1, 10, 'brute', -1);
```

### Collection Methods
*   `default`: Uses PostgreSQL's `standard_planner`.
*   `brute`: Modifies session-level GUCs (`enable_hashjoin`, `enable_mergejoin`, etc.) to force different plan shapes. Possible arms: -1 ~ 63. When arm is -1, pgl_qdataset_collect will run all possible arms.

## Replace components

### Custom Query Planning

pglearned intercepts the PostgreSQL planner hook. You can control its behavior using GUCs.

#### Configuration Variables (GUCs)

*   `pgl.planner_method` (`enum`):
    *   `default` (default): Delegates to `standard_planner`.
    *   `brute`: Enables the brute-force planner logic.

*   `pgl.planner_mode` (`enum`):
    *   `local` (default): Uses local logic (e.g., `pgl.planner_arm`).
    *   `remote`: Connects to an external service to select a plan.

*   `pgl.planner_arm` (`integer`):
    *   Used when `method = 'brute'` and `mode = 'local'`.

*   `pgl.remote_server_url` (`string`):
    *   The endpoint of the gRPC server for `remote` mode (e.g., `http://127.0.0.1:50051`).

#### Remote Inference Mode

To use an external ML model for plan selection:

1.  Set up your gRPC server implementing the `PglRemote` service (see `../proto/pgl_rpc.proto`).
2.  Configure Postgres:
    ```sql
    SET pgl.planner_method = 'brute';
    SET pgl.planner_mode = 'Remote';
    SET pgl.remote_server_url = 'http://localhost:50051';
    ```
3.  Run your query. `pglearned` will:
    *   Generate plans.
    *   Send them to the remote server.
    *   Execute the plan chosen by the server.
