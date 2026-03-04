# nitrocop-filename: example.gemspec
Gem::Specification.new do |spec|
  spec.name = 'example'
  spec.version = '1.0'
  spec.add_dependency 'foo', '~> 1.0'
  spec.add_dependency 'bar'
  spec.authors = ['Author']
  spec.summary = 'An example gem'

  # Dynamic (non-string-literal) arguments should not trigger offense
  common_gemspec.development_dependencies.each do |dep|
    spec.add_development_dependency dep.name, *dep.requirement.as_list
  end

  deps.each { |d| spec.add_development_dependency(d) }

  # Many version args (>2 extra) should not trigger offense per RuboCop's NodePattern limit
  s.add_development_dependency('simplecov', '~> 0.17', '!= 0.18.0', '!= 0.18.1', '!= 0.18.2', '!= 0.18.3', '!= 0.18.4',
                               '!= 0.18.5', '!= 0.19.0', '!= 0.19.1')

  # .freeze suffixed strings are (send (str ...) :freeze) in AST, not bare (str ...),
  # so RuboCop's NodePattern doesn't match them
  s.add_development_dependency(%q<rails>.freeze, ["~> 4.2"])
  s.add_development_dependency('rspec'.freeze, '~> 3.0')
  s.add_development_dependency("rake".freeze)
end
