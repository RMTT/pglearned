import psycopg
from typing import Iterator, Tuple, Dict, Any


class PglClient:
    def __init__(self, dburl: str):
        """
        Initialize the PglClient with a database connection string.

        Args:
            dburl: libpq connection string (e.g., "postgresql://user:password@host:port/dbname")
        """
        self.dburl = dburl

    def qdataset_collect(
        self,
        dataset_name: str,
        offset: int = -1,
        limit: int = 10,
        method: str = "default",
        arm: int = -1,
    ) -> Iterator[Tuple[int, Dict[str, Any]]]:
        """
        Call the pgl_qdataset_collect UDF to fetch query plans.

        Args:
            dataset_name: Name of the dataset to run.
            offset: Offset in the dataset (default -1 uses current_pos).
            limit: Number of queries to run (default 10).
            method: Planner method to use (default 'default').
            arm: Planner arm to use (default -1 for all/iterator).

        Yields:
            Tuple of (id, plan), where plan is a dictionary (JSON).
        """
        # Connect to the database
        with psycopg.connect(self.dburl) as conn:
            with conn.cursor() as cur:
                # Execute the UDF
                # The UDF signature is: pgl_qdataset_collect(dataset_name text, offset int8, limit int8, method text, arm int4)
                cur.execute(
                    "SELECT id, plan FROM pgl_qdataset_collect(%s, %s, %s, %s, %s)",
                    (dataset_name, offset, limit, method, arm),
                )

                # Iterate over results
                for row in cur:
                    yield row[0], row[1]
