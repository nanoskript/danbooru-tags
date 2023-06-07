import gzip
import sys
import json
from pathlib import Path

from sqlitedict import SqliteDict


def main():
    json_gz_path = Path(sys.argv[1])
    sqlite_path = Path(sys.argv[2])

    # Load JSON.
    with gzip.open(Path(json_gz_path), "rb") as f:
        data = json.load(f)

    # Write table.
    with SqliteDict(sqlite_path) as db:
        db.update(data)
        db.commit()


if __name__ == '__main__':
    main()
