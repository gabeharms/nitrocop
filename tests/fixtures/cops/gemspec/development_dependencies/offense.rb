# nitrocop-filename: example.gemspec
Gem::Specification.new do |spec|
  spec.add_development_dependency 'rspec', '~> 3.0'
       ^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/DevelopmentDependencies: Specify development dependencies in `Gemfile` instead of gemspec.
  spec.add_development_dependency 'rubocop'
       ^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/DevelopmentDependencies: Specify development dependencies in `Gemfile` instead of gemspec.
  s.add_development_dependency 'simplecov'
    ^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/DevelopmentDependencies: Specify development dependencies in `Gemfile` instead of gemspec.

  # Percent string literals should also be detected
  s.add_development_dependency(%q<erubis>, ["~> 2.7.0"])
    ^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/DevelopmentDependencies: Specify development dependencies in `Gemfile` instead of gemspec.
  s.add_development_dependency(%q(minitest))
    ^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/DevelopmentDependencies: Specify development dependencies in `Gemfile` instead of gemspec.
  s.add_development_dependency %Q[rake]
    ^^^^^^^^^^^^^^^^^^^^^^^^^^ Gemspec/DevelopmentDependencies: Specify development dependencies in `Gemfile` instead of gemspec.
end
