# stroemung python

This directory contains Python code used to generate test data
from the [`NaSt2D`][nast2d] CFD solver.

Steps:

* Download and compile [`NaSt2D`][nast2d].
* Install [`just`][just] if you don't have it already.
* Install [`pip-tools`][pip-tools] if you don't have it already.
* Install Python dependencies using `just sync`.
* Run `generate_test_data.py`.

## Requirements

### NaSt2D
You will need to
compile `NaSt2D` and provide the path to its `run` executable to
`generate_test_data.py`.

### Python dependencies

`generate_test_data.py` has runtime and development dependencies listed in
`requirements.txt` and `requirements-dev.txt`, which
can be directly installed via `pip`. However, it is easier to use `just sync`
as explained below.

Also, see the note on virtual environments below to learn how to not create
weird issues by installing Python packages globally.

#### just

You will need to install [`just`][just] to run the `justfile`. You can do so via
your system package manager or the `just` releases page.


#### pip-tools

The requirements are managed by `pip-tools`
and generated from `requirements.in` and `requirements-dev.in` respectively.
You will first need to install `pip-tools` into your environment.

```sh
pip install pip-tools
```

#### Installing Dependencies

If you want to run the software, you will need to install or update the
dependencies in your environment. This is called "syncing." To do this, run:

```sh
just sync
```



#### Virtual Environments

It's recommended to install all dependencies into a virtual environment, so as
not to pollute system libraries. Python packaging is unfortunately complicated
and moves relatively fast, so I won't specify a canonical solution. However, I
can suggest two ideas:

##### venv

Modern Python interpreters include the `venv` module, which lets you run

```sh
python3 -m venv venv
source venv/bin/activate
```

to get a `venv` with all the `pip` paths set up correctly. If you are using
a system Python and this doesn't work, make sure you have something like
[`python-venv`][python-venv] installed. Additionally, don't forget to run

```sh
deactivate
```

when you're done.

##### direnv

The major downside of using `venv` directly is you have to remember to
`activate` and `deactivate` it all the time. [`direnv`][direnv] is a wonderful
piece of software that, among other things, can handle this for you
automatically. Assuming you have `direnv` installed and enabled in your shell,
run:

```sh
# You probably want to do this inside of `stroemung/` or `stroemung/python/`,
# not your home directory or something.
# You may have to run `direnv allow` afterward to enable this.
echo "layout python" >> .envrc
```

Whenever you `cd` into this directory, `direnv` will automatically set up and/or
activate Python virtual environment specific to that directory. It will also
automatically deactivate it when you `cd` out. Note that the Python version is
the one specified in `.python-version`, so you may need to edit that file to use
what you have installed, or alternately install that version from
[`pyenv`][pyenv]. It seems that [`uv`][uv] does all of this even better, maybe
this should switch to that at some point.

### Running the software

The software can be run as such:

```sh
./generate_test_data.py
```

### Developing

#### Updating dependencies

If you are working on the software and want to update the dependencies specified
in `requirements.txt` and `requirements-dev.txt`, run:

```sh
just update
# You will have to run `just sync` to actually install the new dependencies.
just sync
```

There is also the convenience command `update-sync`:

```sh
just update-and-sync
```

#### Testing

Tests can be run with `pytest`, or `just test`.

#### Python version

The version of Python used by CI is specified by `.python-version`.

[direnv]: https://direnv.net/
[just]: https://github.com/casey/just
[nast2d]: https://ins.uni-bonn.de/content/software-nast2d
[pip-tools]: https://github.com/jazzband/pip-tools
[pyenv]: https://github.com/pyenv/pyenv
[python-venv]: https://packages.ubuntu.com/search?keywords=python3-venv
[uv]: https://docs.astral.sh/uv/