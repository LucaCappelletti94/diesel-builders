#!/usr/bin/env python3
"""Require executed coverage for the derive parser and expansion pipeline."""

import argparse
from pathlib import Path


def check(report: Path) -> None:
    required = {
        "diesel-builders-derive/src/index.rs",
        "diesel-builders-derive/src/table_model.rs",
        "diesel-builders-derive/src/table_model/attribute_parsing.rs",
    }
    lines = {name: {} for name in required}
    current = None
    for row in report.read_text().splitlines():
        if row.startswith("SF:"):
            source = row[3:].replace("\\", "/")
            current = next((name for name in required if source.endswith("/" + name) or source == name), None)
        elif current is not None and row.startswith("DA:"):
            number, count, *_ = row[3:].split(",")
            number, count = int(number), int(count)
            lines[current][number] = lines[current].get(number, 0) + count
        elif row == "end_of_record":
            current = None

    missing = []
    for name, counts in sorted(lines.items()):
        covered = sum(count > 0 for count in counts.values())
        print(f"{name} {covered}/{len(counts)} lines executed")
        if not covered:
            missing.append(name)
    if missing:
        raise SystemExit("Missing executed derive coverage for " + ", ".join(missing))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("report", type=Path)
    check(parser.parse_args().report)
