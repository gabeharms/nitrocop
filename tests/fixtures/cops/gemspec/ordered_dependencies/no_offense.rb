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

# Dependencies with .freeze on string arg — RuboCop ignores these
Gem::Specification.new do |s|
  if Gem::Version.new(Gem::VERSION) >= Gem::Version.new('1.2.0') then
    s.add_runtime_dependency(%q<rake>.freeze, [">= 0"])
    s.add_runtime_dependency(%q<git>.freeze, [">= 1.2.5"])
    s.add_runtime_dependency(%q<nokogiri>.freeze, [">= 1.5.10"])
  else
    s.add_dependency(%q<rake>.freeze, [">= 0"])
    s.add_dependency(%q<git>.freeze, [">= 1.2.5"])
    s.add_dependency(%q<nokogiri>.freeze, [">= 1.5.10"])
  end
end

# Sort key: _ and - are treated equivalently (both removed for comparison)
Gem::Specification.new do |spec|
  spec.add_dependency "cfn_response"
  spec.add_dependency "cfn-status"
end

# Sort key: rspec_junit_formatter before rspec-rails (underscore and hyphen equivalent)
Gem::Specification.new do |s|
  s.add_development_dependency 'rspec_junit_formatter'
  s.add_development_dependency 'rspec-rails'
end

# Sort key: selenium_statistics before selenium-webdriver
Gem::Specification.new do |s|
  s.add_development_dependency 'selenium_statistics'
  s.add_development_dependency 'selenium-webdriver'
end
