from .adapter import PglAdapter
from .server import run_server

__all__ = ["PglClient", "PglAdapter", "run_server"]


def __getattr__(name):
    if name == "PglClient":
        from .client import PglClient

        return PglClient

    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
