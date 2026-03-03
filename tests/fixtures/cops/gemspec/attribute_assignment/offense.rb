# nitrocop-filename: example.gemspec
# Inconsistent hash assignment (metadata uses both direct and indexed)
Gem::Specification.new do |spec|
  spec.metadata = { 'key-0' => 'value-0' }
  spec.metadata['key-1'] = 'value-1'
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/AttributeAssignment: Use consistent style for Gemspec attributes assignment.
  spec.metadata['key-2'] = 'value-2'
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/AttributeAssignment: Use consistent style for Gemspec attributes assignment.
end

# Inconsistent array assignment (authors uses both direct and indexed)
Gem::Specification.new do |spec|
  spec.authors = %w[author-0 author-1]
  spec.authors[2] = 'author-2'
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/AttributeAssignment: Use consistent style for Gemspec attributes assignment.
end
