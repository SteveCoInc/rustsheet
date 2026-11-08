
project = 'RustSheet'
author = 'RustSheet contributors'

release = '0.1.0'
version = '0.1'

extensions = ['sphinx.ext.autodoc', 'sphinx.ext.napoleon']

templates_path = ['_templates']

exclude_patterns = []

html_theme = 'furo'
html_title = 'RustSheet Python'
html_static_path = ['_static']
html_css_files = ['custom.css']

html_theme_options = {
    'light_css_variables': {
        'color-brand-primary': '#d68742',
        'color-brand-content': '#d68742',
        'color-brand-visited': '#d68742',
        'color-background-primary': '#fcfcfc',
    },
    'dark_css_variables': {
        'color-brand-primary': '#f2994a',
        'color-brand-content': '#f2994a',
        'color-brand-visited': '#f2994a',
    },
}
