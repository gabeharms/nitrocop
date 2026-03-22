# nitrocop-filename: example.gemspec
Gem::Specification.new do |spec|
  spec.name = 'example'
  spec.version = common_gemspec.version
  spec.required_ruby_version = common_gemspec.required_ruby_version
  spec.summary = 'A gem with dynamic version from method call'
end
