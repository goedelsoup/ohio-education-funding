"""CLI: rebuild the committed fixtures from the department's published sources.

    python3 -m ohio_edfund_extract --list
    python3 -m ohio_edfund_extract --rebuild
    python3 -m ohio_edfund_extract --rebuild --refresh   # re-download first

A thin shell. Everything worth testing is in :mod:`.fixtures` and :mod:`.xlsx`.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from .fixtures import REPO, write_all
from .sources import SOURCES, RetrievalError, ensure_readable
from .xlsx import open_workbook


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(prog="ohio_edfund_extract", description=__doc__)
    p.add_argument("--list", action="store_true", help="show the source registry and exit")
    p.add_argument("--rebuild", action="store_true", help="rebuild the committed fixtures")
    p.add_argument("--refresh", action="store_true", help="re-download before rebuilding")
    p.add_argument("--cache", type=Path, default=None, help="override the download cache")
    p.add_argument("--repo", type=Path, default=None, help="override the repository root")
    args = p.parse_args(argv)

    if args.list or not args.rebuild:
        print(f"{'key':<20} {'catalog node':<34} source")
        for s in SOURCES.values():
            print(f"{s.key:<20} {s.catalog:<34} {s.filename}")
            if s.note:
                print(f"{'':<20} {'':<34} {s.note}")
        if not args.rebuild:
            print("\nPass --rebuild to regenerate the fixtures.")
        return 0

    try:
        fy27_path = ensure_readable(SOURCES["fy27-calculator"], args.cache, args.refresh)
        cupp_path = ensure_readable(SOURCES["cupp-fy24"], args.cache, args.refresh)
    except RetrievalError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    root = args.repo or REPO
    with open_workbook(str(fy27_path)) as fy27, open_workbook(str(cupp_path)) as cupp:
        counts = write_all(fy27, cupp, repo=root)

    for path, n in counts.items():
        print(f"{n:>5} rows  {path}")
    print("\nNow re-run the Rust gate; the fixtures are pinned by tests and a source change "
          "that moves a finding will fail there rather than pass silently.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
