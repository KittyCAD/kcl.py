#!/usr/bin/env python3
import os

import kcl

# Get the path to this script's parent directory.
kcl_dir_file_path = os.path.join(
    os.path.dirname(os.path.realpath(__file__)), "..", "files"
)


def test_kcl_execute_and_snapshot():
    # Read from a file.
    with open(os.path.join(kcl_dir_file_path, "lego.kcl"), "w") as f:
        image_bytes = kcl.execute_and_snapshot(str(f), kcl.UnitLength, 'jpeg')
        assert image_bytes is not None
        assert len(image_bytes) > 0
