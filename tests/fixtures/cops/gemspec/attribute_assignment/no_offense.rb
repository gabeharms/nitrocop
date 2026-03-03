# nitrocop-filename: example.gemspec
# Only indexed hash assignment
Gem::Specification.new do |spec|
  spec.metadata['key-0'] = 'value-0'
  spec.metadata['key-1'] = 'value-1'
end

# Only direct hash assignment
Gem::Specification.new do |spec|
  spec.metadata = { 'key' => 'value' }
  spec.metadata = { 'key' => 'value' }
end

# Only indexed array assignment
Gem::Specification.new do |spec|
  spec.authors[0] = 'author-0'
  spec.authors[1] = 'author-1'
end

# Only direct array assignment
Gem::Specification.new do |spec|
  spec.authors = %w[author-1 author-2]
  spec.authors = %w[author-1 author-2]
end

# Duplicate direct assignments are NOT this cop's concern
Gem::Specification.new do |spec|
  spec.name = 'example'
  spec.version = '1.0'
  spec.name = 'other'
  spec.summary = 'Summary'
  spec.summary = 'Other summary'
end

# Consistent per attribute even if different styles overall
Gem::Specification.new do |spec|
  spec.metadata = { 'key' => 'value' }
  spec.authors[0] = 'author-0'
  spec.authors[1] = 'author-1'
  spec.authors[2] = 'author-2'
end

# Normal gemspec with various attributes
Gem::Specification.new do |spec|
  spec.name = 'example'
  spec.version = '1.0'
  spec.summary = 'An example gem'
  spec.authors = ['Author']
  spec.metadata['homepage_uri'] = 'https://example.com'
  spec.metadata['source_code_uri'] = 'https://github.com/example'
end
