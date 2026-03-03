# nitrocop-filename: example.gemspec
Gem::Specification.new do |spec|
  spec.add_dependency 'aaa', '~> 1.0'
  spec.add_dependency 'bbb', '~> 2.0'
  spec.add_dependency 'ccc'

  spec.add_development_dependency 'alpha'
  spec.add_development_dependency 'beta'
end

# Blank lines separate groups — each group is independently sorted
Gem::Specification.new do |spec|
  spec.add_dependency "core-watch", "~> 0.0.1"

  spec.add_dependency "async-http", "~> 0.54.1"
end

# Multiple blank-line-separated groups, each sorted
Gem::Specification.new do |spec|
  spec.add_dependency 'foo'
  spec.add_dependency 'zoo'

  spec.add_dependency 'alpha'
  spec.add_dependency 'beta'

  spec.add_dependency 'delta'
end
