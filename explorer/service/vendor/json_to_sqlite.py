import sys
import json
from pathlib import Path

from sqlitedict import SqliteDict


def main():
    json_path = Path(sys.argv[1])
    sqlite_path = Path(sys.argv[2])

    # Load JSON.
    with open(Path(json_path), "rb") as f:
        data = json.load(f)

    # Write table.
    with SqliteDict(sqlite_path) as db:
        db.update(data)
        db.commit()


if __name__ == '__main__':
    main()
