#!/usr/bin/env python3
import os

import kcl
import pytest

# Get the path to this script's parent directory.
kcl_dir_file_path = os.path.join(
    os.path.dirname(os.path.realpath(__file__)), "..", "files"
)


@pytest.mark.asyncio
async def test_kcl_execute_and_snapshot():
    # Read from a file.
    with open(os.path.join(kcl_dir_file_path, "lego.kcl"), "r") as f:
        code = str(f.read())
        print(code)
        image_bytes = await kcl.execute_and_snapshot(
            code, kcl.UnitLength.Mm, kcl.ImageFormat.Jpeg
        )
        assert image_bytes is not None
        assert len(image_bytes) > 0
