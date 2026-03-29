import json

from pgl import PglAdapter, run_server


class ConstantCardinalityEstimator(PglAdapter):
    def __init__(self, scale=1.0):
        self.scale = scale

    def choose_plan(self, plans):
        return 0

    def cardinality_estimate(self, rel_opts):
        estimates = []
        for rel_opt in rel_opts:
            payload = json.loads(rel_opt)
            rows = payload.get("rows", 0)
            estimates.append(max(0, int(rows * self.scale)))
        return estimates


if __name__ == "__main__":
    run_server(ConstantCardinalityEstimator())
