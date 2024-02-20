Getting Started
===============

Installation
------------

Bourse can be installed from pip

.. code-block:: bash

   pip install bourse

Building from Source
--------------------

Building from source requires rust and cargo to be installed
see `here <https://doc.rust-lang.org/cargo/getting-started/installation.html>`_
for instructions.

Bourse uses `hatch <https://hatch.pypa.io/latest/>`_ for dependency management,
see the `hatch docs <https://hatch.pypa.io/latest/install/>`_ for installation
instructions.

Bourse can then be built using

.. code-block:: bash

   hatch run dev:build

Direct Dependency
-----------------

The latest version of bourse can be added as a github dependency in your
projects ``pyproject.toml``

.. code-block:: bash

   dependencies = [
      "bourse@git+ssh://git@github.com/zombie-einstein/bourse.git"
   ]

but also requires that maturin is added as a build requirement,
for example

.. code-block:: bash

   [build-system]
   requires = ["setuptools >= 61.0", "maturin>=1.2,<2.0"]
   build-backend = "setuptools.build_meta"
