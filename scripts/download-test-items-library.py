#!/usr/bin/env python3
"""Download all collections from the Zotero Test Items Library (group 2205533).

Handles API pagination (max 100 items per request) and merges pages into
a single CSL JSON file per collection.

Usage:
    python3 scripts/download-test-items-library.py [output_dir]
    
    output_dir defaults to tests/fixtures/test-items-library/
"""

import json
import os
import sys
import time
import urllib.request

GROUP_ID = "2205533"
BASE_URL = f"https://api.zotero.org/groups/{GROUP_ID}"
PAGE_SIZE = 100

# Collection key → output filename
COLLECTIONS = {
    "5V67EPX3": "chicago-18th",
    "MR2N872S": "apa-7th",
    "9RTDRMPL": "apa-test",
    "DMKBHP6P": "apa-6th",
    "L96HMJXY": "mla-9th",
    "D9MVMAFF": "mhra-4th",
    "499SCKUW": "new-harts-rules-2nd",
    "AC7ETLN4": "oscola-4th",
    "QW4JGG9G": "ama-11th",
    "EJ39UBC4": "csl-repository-test",
    "424HZVK3": "zotero-style-repository",
    "ECHCPXZN": "zotero-repository-proposed",
}


def fetch_collection(collection_key: str, name: str) -> list[dict]:
    """Fetch all items from a collection, handling pagination."""
    all_items = []
    start = 0

    while True:
        url = (
            f"{BASE_URL}/collections/{collection_key}/items"
            f"?format=csljson&limit={PAGE_SIZE}&start={start}&v=3"
        )
        req = urllib.request.Request(url)
        req.add_header("Zotero-API-Version", "3")

        with urllib.request.urlopen(req) as resp:
            total = int(resp.headers.get("Total-Results", 0))
            data = json.load(resp.read().decode("utf-8"))
            
            if isinstance(data, list):
                items = data
            elif isinstance(data, dict):
                items = data.get("items", [])
            else:
                raise TypeError(f"Unexpected CSL JSON response type: {type(data)!r}")
                
            all_items.extend(items)

            if start == 0:
                print(f"  {name}: {total} total items")

        start += PAGE_SIZE
        if start >= total:
            break
        time.sleep(0.5)  # Be polite to the API

    # Deduplicate by id
    seen = set()
    unique = []
    for item in all_items:
        iid = item.get("id")
        if iid not in seen:
            seen.add(iid)
            unique.append(item)

    return unique


def main():
    output_dir = sys.argv[1] if len(sys.argv) > 1 else "tests/fixtures/test-items-library"
    os.makedirs(output_dir, exist_ok=True)

    summary = {}

    for key, name in COLLECTIONS.items():
        print(f"Downloading: {name} ({key})...")
        try:
            items = fetch_collection(key, name)
            output_path = os.path.join(output_dir, f"{name}.json")
            with open(output_path, "w", encoding="utf-8") as f:
                json.dump({"items": items}, f, indent=2, ensure_ascii=False)
            summary[name] = len(items)
            print(f"  → {len(items)} items saved to {output_path}")
        except Exception as e:
            print(f"  ✗ Error downloading {name}: {e}", file=sys.stderr)
            summary[name] = f"ERROR: {e}"

    print("\n=== Summary ===")
    for name, count in summary.items():
        print(f"  {name}: {count}")


if __name__ == "__main__":
    main()
