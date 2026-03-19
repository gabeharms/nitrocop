# nitrocop-filename: example.gemspec
# nitrocop-expect: 2:31 Gemspec/RequiredRubyVersion: `required_ruby_version` and `TargetRubyVersion` (4.0, which may be specified in .rubocop.yml) should be equal.
Gem::Specification.new do |spec|
  spec.required_ruby_version = [
    ">= #{Datadog::CI::VERSION::MINIMUM_RUBY_VERSION}",
    "< #{Datadog::CI::VERSION::MAXIMUM_RUBY_VERSION}"
  ]
  spec.name = "datadog-ci"
  spec.version = "1.0"
  spec.summary = "A gem with array interpolation version"
end
