YAML.load_file('config.yml')
YAML.safe_load_file('data.yml')
YAML.load(content)
YAML.safe_load(string_data)
x = YAML.parse_file('input.yml')
y = File.read('data.txt')

# File.read with encoding keyword arg - not replaceable with safe_load_file
YAML.safe_load(File.read(filepath, encoding: Encoding::UTF_8))
