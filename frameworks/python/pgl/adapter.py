from abc import ABC, abstractmethod
from typing import List, Dict, Any


class PglAdapter(ABC):
    """
    Abstract base class for implementing a pglearned adapter.
    """

    @abstractmethod
    def choose_plan(self, plans: List[Dict[str, Any]]) -> int:
        """
        Choose the best query plan from a list of candidates.

        Args:
            plans: A list of query plans (parsed as dictionaries).

        Returns:
            The index of the chosen plan (0-based).
        """
        pass
