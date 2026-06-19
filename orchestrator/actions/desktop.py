import os
import glob
from typing import Optional


COMMON_IMAGE_PATHS = [
    os.path.expanduser("~/Desktop"),
    os.path.expanduser("~/Pictures"),
    os.path.expanduser("~/Downloads"),
]

IMAGE_EXTENSIONS = {"*.jpg", "*.jpeg", "*.png", "*.gif", "*.webp", "*.bmp", "*.tiff", "*.heic"}


class DesktopScanner:
    def __init__(self):
        self.scanned_paths = {}

    async def list_images(self, path: Optional[str] = None) -> dict:
        search_paths = [path] if path else COMMON_IMAGE_PATHS
        found = []

        for search_path in search_paths:
            if not os.path.isdir(search_path):
                continue
            for ext in IMAGE_EXTENSIONS:
                pattern = os.path.join(search_path, "**", ext)
                for filepath in glob.glob(pattern, recursive=True):
                    if os.path.isfile(filepath):
                        size = os.path.getsize(filepath)
                        found.append({
                            "path": filepath,
                            "name": os.path.basename(filepath),
                            "size_bytes": size,
                            "size_kb": round(size / 1024, 1),
                            "directory": os.path.dirname(filepath),
                        })

        found.sort(key=lambda x: x["size_bytes"], reverse=True)
        return {"success": True, "images": found, "count": len(found), "paths_searched": search_paths}

    async def get_desktop_paths(self) -> dict:
        paths = {}
        for p in COMMON_IMAGE_PATHS:
            paths[p] = os.path.isdir(p)
        return {"success": True, "paths": paths}
