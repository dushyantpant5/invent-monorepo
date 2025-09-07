# Traefik/render_dynamic.py
# Renders dynamic.yml.template by substituting ${VAR} and ${VAR:-default}
# Usage: python3 render_dynamic.py > dynamic.yml

import os
import re
import sys

TEMPLATE = os.path.join(os.path.dirname(__file__), "dynamic.yml.template")

with open(TEMPLATE, "r", encoding="utf-8") as fh:
    tmpl = fh.read()

pattern = re.compile(
    r'\$\{(?P<var>[A-Za-z_][A-Za-z0-9_]*)' +
    r'(?:\:-(?P<def>[^}]*))?\}'
)


def repl(m):
    var = m.group("var")
    default = m.group("def") if m.group("def") is not None else ""
    return os.environ.get(var, default)


sys.stdout.write(pattern.sub(repl, tmpl))
