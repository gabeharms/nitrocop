YAML.load(data)
^^^^^^^^^^^^^^^ Security/YAMLLoad: Prefer using `YAML.safe_load` over `YAML.load`.

::YAML.load(payload)
^^^^^^^^^^^^^^^^^^^^ Security/YAMLLoad: Prefer using `YAML.safe_load` over `YAML.load`.

YAML.load(File.read("config.yml"))
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Security/YAMLLoad: Prefer using `YAML.safe_load` over `YAML.load`.
