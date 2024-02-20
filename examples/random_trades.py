import bourse


class RandomAgent:
    def __init__(self, i, price_range):
        self.i = i
        self.price_range = price_range
        self.order_id = None

    def update(self, rng, env):
        # Place an order if not one live
        if self.order_id is None:
            price = rng.integers(*self.price_range)
            side = bool(rng.choice([True, False]))
            env.place_order(side, 10, self.i, price=price)
        # Cancel live order
        else:
            env.cancel_order(self.order_id)
            self.order_id = None


def run(seed: int, n_steps: int, n_agents):

    agents = [RandomAgent(i, (10, 100)) for i in range(n_agents)]
    env = bourse.core.StepEnv(seed, 0, 100_000)

    market_data = bourse.step_sim.run(env, agents, n_steps, seed)

    return market_data


if __name__ == "__main__":
    run(101, 200, 100)
