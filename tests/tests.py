#!/usr/bin/env python3
import os

import kcl
import pytest

# Get the path to this script's parent directory.
kcl_dir_file_path = os.path.join(
    os.path.dirname(os.path.realpath(__file__)), "..", "files"
)


@pytest.mark.asyncio
async def test_kcl_execute():
    # Read from a file.
    with open(os.path.join(kcl_dir_file_path, "lego.kcl"), "r") as f:
        code = str(f.read())
        assert code is not None
        assert len(code) > 0
        await kcl.execute(code, kcl.UnitLength.Mm)


@pytest.mark.asyncio
async def test_kcl_execute_and_snapshot():
    # Read from a file.
    with open(os.path.join(kcl_dir_file_path, "lego.kcl"), "r") as f:
        code = str(f.read())
        assert code is not None
        assert len(code) > 0
        image_bytes = await kcl.execute_and_snapshot(
            code, kcl.UnitLength.Mm, kcl.ImageFormat.Jpeg
        )
        assert image_bytes is not None
        assert len(image_bytes) > 0


@pytest.mark.asyncio
async def test_kcl_execute_and_export():
    # Read from a file.
    with open(os.path.join(kcl_dir_file_path, "lego.kcl"), "r") as f:
        code = str(f.read())
        assert code is not None
        assert len(code) > 0
        files = await kcl.execute_and_export(
            code, kcl.UnitLength.Mm, kcl.FileExportFormat.Step
        )
        assert files is not None
        assert len(files) > 0
        assert files[0] is not None
        name = files[0].name
        contents = files[0].contents
        assert name is not None
        assert len(name) > 0
        assert contents is not None
        assert len(contents) > 0


def test_kcl_format():
    # Read from a file.
    with open(os.path.join(kcl_dir_file_path, "lego.kcl"), "r") as f:
        code = str(f.read())
        assert code is not None
        assert len(code) > 0
        formatted_code = kcl.format(code)
        assert formatted_code is not None
        assert len(formatted_code) > 0


def test_kcl_lint():
    # Read from a file.
    with open(os.path.join(kcl_dir_file_path, "box_with_linter_errors.kcl"), "r") as f:
        code = str(f.read())
        assert code is not None
        assert len(code) > 0
        lints = kcl.lint(code)
        assert lints is not None
        assert len(lints) > 0
        description = lints[0].description
        assert description is not None
        assert len(description) > 0
        finding = lints[0].finding
        assert finding is not None
        finding_title = finding.title
        assert finding_title is not None
        assert len(finding_title) > 0
