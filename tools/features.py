import json
import tomllib
from itertools import chain, combinations
from os import environ
from pathlib import Path

cargo_toml = Path.cwd() / "Cargo.toml"
manifest = tomllib.load(cargo_toml.open("rb"))

features = manifest.get("features", {}).keys()
features_powerset = list(chain.from_iterable(combinations(features, r) for r in range(len(features)+1)))

result = json.dumps([','.join(feature_set) for feature_set in features_powerset])

if (path := environ.get("GITHUB_OUTPUT")) is not None:
    with open(path, "a") as output:
        output.write(f"features={result}\n")
else:
    print(result)
