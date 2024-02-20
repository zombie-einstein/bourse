import pkg_resources

project = "Bourse"
copyright = "2024, zombie-einstein"
author = "zombie-einstein"
release = pkg_resources.get_distribution("bourse").version

extensions = [
    "sphinx.ext.napoleon",
    "sphinx.ext.autosectionlabel",
    "sphinx_immaterial",
    "sphinx_immaterial.apidoc.python.apigen",
    "sphinx.ext.intersphinx",
    "sphinx.ext.doctest",
]

napoleon_google_docstring = False
napoleon_numpy_docstring = True

napoleon_include_init_with_doc = True
napoleon_include_private_with_doc = False
napoleon_include_special_with_doc = False
napoleon_use_admonition_for_examples = False
napoleon_use_admonition_for_notes = True
napoleon_use_admonition_for_references = False
napoleon_use_ivar = False
# napoleon_use_param = True
napoleon_use_rtype = False
napoleon_preprocess_types = True
napoleon_attr_annotations = True

add_module_names = False

exclude_patterns = []

intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "numpy": ("https://numpy.org/doc/stable", None),
    "pandas": ("https://pandas.pydata.org/docs/", None),
}

html_title = "Bourse"
html_theme = "sphinx_immaterial"

html_theme_options = {
    "repo_url": "https://github.com/zombie-einstein/bourse",
    "icon": {
        "repo": "fontawesome/brands/github",
    },
    "palette": {
        "scheme": "slate",
        "primary": "teal",
    },
    "toc_title_is_page_title": True,
}

python_apigen_order_tiebreaker = "alphabetical"

python_apigen_modules = {
    "bourse.data_processing": "pages/generated/data_processing/",
    "bourse.step_sim": "pages/generated/step_sim/",
    "bourse.core": "pages/generated/core/",
}

python_apigen_default_groups = [
    (r".*data_processing.*", "data_processing"),
    (r".*step_sim.*", "step_sim"),
    (r"class:.*OrderBook.*", "order_book_class"),
    (r"method:.*OrderBook.*", "OrderBook Methods"),
    (r"attribute:.*OrderBook.*", "OrderBook Attributes"),
    (r"class:.*StepEnv.*", "step_env_class"),
    (r"method:.*StepEnv.*", "StepEnv Methods"),
    (r"attribute:.*StepEnv.*", "StepEnv Attributes"),
]
