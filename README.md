# kcl.py
Python bindings to the rust kcl-lib crate.


## Development

We use [maturin](https://github.com/PyO3/maturin) for this project.

You can either download binaries from the [latest release](https://github.com/PyO3/maturin/releases/latest) or install it with [pipx](https://pypa.github.io/pipx/):

```shell
pipx install maturin
```

> [!NOTE]
>
> `pip install maturin` should also work if you don't want to use pipx.

There are four main commands:

- `maturin publish` builds the crate into python packages and publishes them to pypi.
- `maturin build` builds the wheels and stores them in a folder (`target/wheels` by default), but doesn't upload them. It's possible to upload those with [twine](https://github.com/pypa/twine) or `maturin upload`.
- `maturin develop` builds the crate and installs it as a python module directly in the current virtualenv. Note that while `maturin develop` is faster, it doesn't support all the feature that running `pip install` after `maturin build` supports.

`pyo3` bindings are automatically detected. 
`maturin` doesn't need extra configuration files and doesn't clash with an existing setuptools-rust or milksnake configuration.

